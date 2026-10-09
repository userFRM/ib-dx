"""A snapshot ends when it is whole, as a gateway makes one whole.

A gateway holds a snapshot until the bid, the ask, the last, the open and the
close have each been stated; on a contract it marks as an option also the
venue's option model (13, or 83 delayed) and the bid's, ask's and last's
computations (10 to 12, or 80 to 82); on a delayed feed also the last trade's
time (88). Ended on the five alone, an option's snapshot reached
`tickSnapshotEnd` before its model, and a program reading the ticker when it
ended read an option with no model.

It sends a snapshot each kind of tick once, the first it holds, and the bid's,
the ask's and the last's sizes only beside their prices.
"""

from ib_dx import Contract, EClient, EWrapper


class Heard(EWrapper):
    def __init__(self):
        super().__init__()
        self.ended = []
        self.times = []
        self.said = []

    def tickPrice(self, req_id, tick_type, price, attrib):
        self.said.append(("price", tick_type, price))

    def tickSize(self, req_id, tick_type, size):
        self.said.append(("size", tick_type, size))

    def tickSnapshotEnd(self, req_id):
        self.ended.append(req_id)

    def tickString(self, req_id, tick_type, value):
        if tick_type in (45, 88):
            self.times.append((req_id, tick_type))


FIVE = dict(bid=10.0, ask=10.5, last=10.2, open=10.0, close=9.9)


def _session():
    heard = Heard()
    client = EClient(heard)
    client._test_connect("DU0000000")
    return client, heard


def test_an_options_snapshot_waits_for_its_computations():
    client, heard = _session()
    option = Contract(
        conId=700001, symbol="SPY", secType="OPT", exchange="SMART", currency="USD",
        lastTradeDateOrContractMonth="20261218", strike=500.0, right="C", multiplier="100",
    )
    client.reqMktData(1, option, "", True, False, [])
    client._test_dispatch_once()
    slot = client._test_watching(1)
    assert slot is not None, "the engine took the request"

    client._test_push_quote(slot, **FIVE)
    client._test_dispatch_once()
    assert heard.ended == [], "an option's snapshot ended without its model"

    client._test_push_option_model(slot, 0.2, 5.0, 500.0)
    client._test_dispatch_once()
    assert heard.ended == [], "an option's snapshot ended without the bid's, ask's and last's computations"


def test_a_delayed_snapshot_waits_for_the_time():
    client, heard = _session()
    client.req_mkt_data_ex(2, Contract(conId=756733, secType="STK", exchange="SMART"), "", True, False, 1)
    client._test_dispatch_once()
    slot = client._test_watching(2)
    assert slot is not None, "the engine took the request"

    client._test_push_quote(slot, timestamp_ns=0, **FIVE)
    client._test_dispatch_once()
    assert heard.ended == [], "a delayed snapshot ended without the time"

    client._test_push_quote(slot, timestamp_ns=1_700_000_000_000_000_000, **FIVE)
    client._test_dispatch_once()
    assert heard.times == [(2, 88)], heard.times
    assert heard.ended == [2], heard.ended


def test_a_snapshot_is_sent_each_kind_once_and_a_size_beside_its_price():
    client, heard = _session()
    client.reqMktData(3, Contract(conId=756733, secType="STK", exchange="SMART"), "", True, False, [])
    client._test_dispatch_once()
    slot = client._test_watching(3)
    assert slot is not None, "the engine took the request"

    client._test_push_quote(slot, bid=10.0, ask=10.5, last=10.25, bid_size=4, ask_size=6, last_size=1)
    client._test_dispatch_once()
    client._test_push_quote(slot, bid=10.125, ask=10.5, last=10.25, bid_size=9, ask_size=6, last_size=1)
    client._test_dispatch_once()
    assert heard.said == [
        ("price", 1, 10.0), ("size", 0, 4.0), ("price", 2, 10.5), ("size", 3, 6.0),
        ("price", 4, 10.25), ("size", 5, 1.0),
    ], heard.said
