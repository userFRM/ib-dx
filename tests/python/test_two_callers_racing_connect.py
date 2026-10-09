"""Only one of two callers racing to connect gets a session.

The refusal and the flag it reads were two steps, so both callers found the
flag clear and both built an engine. The second replaced the first, which went
on running with a live socket and a second logon the account never asked for —
and the venue bumps the older session when that happens, so a caller racing
itself knocks over its own.

The wheel is built free-threaded, so this is not a theoretical interleaving.
"""

import threading

import pytest
from ib_dx import EClient, EWrapper


@pytest.mark.parametrize("attempt", range(20))
def test_only_one_of_two_racing_connects_takes_the_session(attempt):
    errors = []
    lock = threading.Lock()

    class Probe(EWrapper):
        def error(self, req_id, error_time, code, msg, advanced_order_reject_json=""):
            with lock:
                errors.append(code)

    client = EClient(Probe())
    ready = threading.Barrier(2)

    def connect():
        ready.wait()
        client._test_connect("DU111111", True)

    threads = [threading.Thread(target=connect) for _ in range(2)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    # Both calls return normally, as the reference client's connect does; the
    # one that lost the claim is told so on the wrapper, under 501, and there
    # is exactly one of those. Zero would mean both built a session, and the
    # second left the first running with a live socket and a second logon.
    assert errors == [501], errors
