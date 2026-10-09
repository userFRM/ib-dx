"""`reqHistoricalNews` takes `totalResults` as the TWS API's `int`, as a gateway does.

A gateway asks the venue for no more than three hundred headlines and passes a
smaller positive number on as stated. A count below one is refused with 321
"Total results must be > 0" before the venue is asked anything.
"""

import ib_dx
from conftest import refused


class Errors(ib_dx.EWrapper):
    def __init__(self):
        super().__init__()
        self.seen = []

    def error(self, req_id, error_time, code, msg, advanced_order_reject_json=""):
        self.seen.append((req_id, code, msg))


def _client():
    w = Errors()
    c = ib_dx.EClient(w)
    c._test_connect("DU1")
    c._test_set_news_providers(["BRFG"])
    return w, c


def _asked(total_results):
    w, c = _client()
    c.reqHistoricalNews(1, 265598, "BRFG", "", "", total_results, [])
    sent = [cmd for cmd in c._test_take_commands() if "FetchHistoricalNews" in cmd]
    assert len(sent) == 1, sent
    assert w.seen == []
    return sent[0]


def test_more_than_three_hundred_asks_for_three_hundred():
    assert "max_results: 300" in _asked(500)


def test_a_smaller_number_is_passed_on():
    assert "max_results: 7" in _asked(7)


def test_a_number_below_one_is_refused():
    w, c = _client()
    c.reqHistoricalNews(1, 265598, "BRFG", "", "", 0, [])
    c.reqHistoricalNews(2, 265598, "BRFG", "", "", -5, [])
    c.poll()
    assert w.seen == [
        refused(1, 321, "Total results must be > 0"),
        refused(2, 321, "Total results must be > 0"),
    ]
    sent = [cmd for cmd in c._test_take_commands() if "FetchHistoricalNews" in cmd]
    assert not sent, f"the venue was asked: {sent}"
