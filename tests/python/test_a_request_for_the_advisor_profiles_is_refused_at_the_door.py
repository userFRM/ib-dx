"""A request for the advisor's allocation profiles never reaches the wire.

The reference client refuses faData 2 — the allocation profiles — at the
door, under 585 and in its own words, and sends nothing; a session here
stands at a level where that refusal applies. A number naming no partition
travels the other way: the reference client forwards it, and the refusal the
caller hears is the intake's own sentence, not one invented here.
"""

import ibkr_dx
from conftest import refused

FA_PROFILE_UNSUPPORTED = "FA Profile is not supported anymore, use FA Group instead - "


class Errors(ibkr_dx.EWrapper):
    def __init__(self):
        super().__init__()
        self.seen = []

    def error(self, req_id, error_time, code, msg, advanced_order_reject_json=""):
        self.seen.append((req_id, code, msg))


def _client():
    w = Errors()
    c = ibkr_dx.EClient(w)
    c._test_connect("DU1")
    return w, c


def test_a_profile_request_is_refused_with_585_and_sends_nothing():
    w, c = _client()
    c.requestFA(2)
    c.replaceFA(9, 2, "<xml/>")
    c.poll()
    assert w.seen == [
        (-1, 585, FA_PROFILE_UNSUPPORTED),
        (9, 585, FA_PROFILE_UNSUPPORTED),
    ]
    assert not any("AdvisorConfig" in cmd for cmd in c._test_take_commands())


def test_a_request_naming_an_unknown_number_hears_the_intakes_own_refusal():
    w, c = _client()
    c.requestFA(7)
    c.replaceFA(9, 7, "<xml/>")
    c.poll()
    assert w.seen == [
        refused(-1, 321, "Non-existent FA data operation request."),
        refused(9, 321, "Non-existent FA data operation request."),
    ]


def test_a_request_naming_a_partition_is_forwarded():
    w, c = _client()
    c.requestFA(1)
    c.replaceFA(9, 3, "<xml/>")
    c.poll()
    assert w.seen == []
    assert sum("AdvisorConfig" in cmd for cmd in c._test_take_commands()) == 2
