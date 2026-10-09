"""A request returns once the engine has it, and no gauge of its queue is offered.

A TWS call returns once its message is written. Here the message is a command
handed to the engine's loop, and the channel it goes down is unbounded: a call
never waits for the loop to make room, however far behind the loop is.

The engine used to offer the count of what it had not finished with as
`backlog()`. No gateway wire request backs that count and no reference client
defines the call — it named this client's own admission queue — so it is
offered on neither surface; the engine keeps its count to itself.
"""

import threading

import ib_dx


def test_ten_thousand_requests_return():
    c = ib_dx.EClient(ib_dx.EWrapper())
    c._test_connect("T")
    # Nothing takes from the channel behind a test session.
    admitting = threading.Thread(
        target=lambda: [c.cancelHistoricalData(req_id) for req_id in range(1, 10_001)],
        daemon=True,
    )
    admitting.start()
    admitting.join(10)
    assert not admitting.is_alive(), "a request waited for an engine that was not taking"


def test_the_admission_count_is_not_offered():
    assert not hasattr(ib_dx.EClient, "backlog")
