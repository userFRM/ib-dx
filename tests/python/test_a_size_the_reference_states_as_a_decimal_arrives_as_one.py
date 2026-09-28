"""A size the reference states as a Decimal arrives as one.

The reference client decodes every size and quantity it hands a program with
`decode(Decimal, fields)` — its wrapper annotations say `size: Decimal` for a
tick, a print, a book row, a holding, a fill — and a record it builds carries
`UNSET_DECIMAL` until the venue states the field. This client handed over a
float everywhere instead (an int for a bar's volume), so a program written
against the reference raised a TypeError summing sizes into its Decimal
accumulator, and the read loop that called the handler logged the exception
and moved on: every size was silently lost. The values are held fixed-point
here, so each one crosses as its own digits, and an unstated field reads as
UNSET_DECIMAL.

Run: pytest tests/python/test_a_size_the_reference_states_as_a_decimal_arrives_as_one.py -v
"""

import time
from decimal import Decimal

import ibkr_dx
from ibkr_dx import UNSET_DECIMAL


class Heard(ibkr_dx.EWrapper):
    def __init__(self):
        super().__init__()
        self.sizes = []
        self.statuses = []
        self.fills = []
        self.holdings = []
        self.portfolio = []
        self.prints = []
        self.quotes = []
        self.depth = []
        self.ticks = []
        self.buckets = []
        self.bars = []

    def tickSize(self, reqId, tickType, size):
        self.sizes.append(size)

    def orderStatus(self, orderId, status, filled, remaining, avgFillPrice,
                    permId, parentId, lastFillPrice, clientId, whyHeld, mktCapPrice):
        self.statuses.append((filled, remaining))

    def execDetails(self, reqId, contract, execution):
        self.fills.append((execution.shares, execution.cumQty))

    def position(self, account, contract, pos, avgCost):
        self.holdings.append(pos)

    def positionMulti(self, reqId, account, model, contract, pos, avgCost):
        self.holdings.append(pos)

    def updatePortfolio(self, contract, pos, marketPrice, marketValue,
                        averageCost, unrealizedPNL, realizedPNL, accountName):
        self.portfolio.append(pos)

    def tickByTickAllLast(self, reqId, tickType, time, price, size, attrib,
                          exchange, specialConditions):
        self.prints.append(size)

    def tickByTickBidAsk(self, reqId, time, bidPrice, askPrice, bidSize, askSize, attrib):
        self.quotes.append((bidSize, askSize))

    def updateMktDepth(self, reqId, position, operation, side, price, size):
        self.depth.append(size)

    def historicalTicksLast(self, reqId, ticks, done):
        self.ticks.extend(tick.size for tick in ticks)

    def histogramData(self, reqId, items):
        self.buckets.extend(item.size for item in items)

    def historicalData(self, reqId, bar):
        self.bars.append((bar.volume, bar.wap))

    def error(self, *args):
        pass


def test_every_delivered_size_is_the_decimal_the_reference_states():
    heard = Heard()
    client = ibkr_dx.EClient(heard)
    client._test_connect("DU1", True, accounts=["DU1"])
    client._test_set_instrument_count(8)
    client._test_map_instrument(1, 7)
    client._test_map_instrument(0, 0)
    client._test_push_quote(7, bid=412.0, bid_size=100, ask_size=200, last_size=300)
    client._test_push_tbt_trade(0, 412.55, 100, "NYSE", kind="AllLast")
    client._test_push_tbt_quote(0, 412.5, 412.6, 100, 200)
    client._test_push_depth(7, 0, "", 0, 1, 100.0, 5.0)
    client._test_push_histogram(7, 101.5, 42)
    client._test_push_historical_ticks(7, ["20260812 10:00:00"])
    client._test_push_historical_data(
        1,
        [
            ("20260812-13:30:00", 1.0, 2.0, 0.5, 1.5, 100),
            # A crypto bar's volume is a fraction of a coin: the digits the
            # venue stated must survive to the Decimal, not round to a count.
            ("20260812-13:35:00", 1.0, 2.0, 0.5, 1.5, 0.5342),
        ],
        True,
        "US/Eastern",
    )
    client._test_push_venue_order(86, "SPY", "BUY", 1, 100.0)
    client._test_push_fill(0, 86, "BUY", 100.0, 3, 2)
    client._test_set_position(756733, 9, 30)
    client._test_set_position(756733, 9, 30, account="DU1")
    client._test_set_account(net_liquidation=100, daily_pnl=1, account="DU1")
    client._test_finish_account_download("DU1")
    client.reqPositions()
    client.reqPositionsMulti(3, "DU1", "")
    client.reqAccountUpdates(True, "DU1")
    client.poll()
    # The engine names the holdings' contracts before it answers the ask,
    # and answers after it: one holding is the ask's answer, the other the
    # multi-account ask's.
    deadline = time.monotonic() + 5
    while heard.holdings.count(Decimal("9")) < 2 and time.monotonic() < deadline:
        client._test_dispatch_once()
        time.sleep(0.05)

    named = [
        ("tickSize", heard.sizes),
        ("orderStatus", heard.statuses),
        ("execDetails", heard.fills),
        ("position", heard.holdings),
        ("updatePortfolio", heard.portfolio),
        ("tickByTickAllLast", heard.prints),
        ("tickByTickBidAsk", heard.quotes),
        ("updateMktDepth", heard.depth),
        ("historicalTicksLast", heard.ticks),
        ("histogramData", heard.buckets),
        ("historicalData", heard.bars),
    ]
    for name, values in named:
        assert values, f"{name} reached nothing"
        flat = [v for pair in values for v in (pair if isinstance(pair, tuple) else (pair,))]
        for value in flat:
            assert isinstance(value, Decimal), (
                f"{name}: the reference states Decimal, got "
                f"{type(value).__name__} {value!r}"
            )

    assert Decimal("100") in heard.sizes
    assert heard.statuses == [(Decimal("3"), Decimal("2"))]
    # Shares are the print's; cumQty is what the report stated, and this
    # report stated nothing past its own defaults.
    assert heard.fills == [(Decimal("3"), Decimal("0"))]
    # One from the ask's answer, one from the multi-account ask; the feed
    # may state the moved holding again beside them.
    assert heard.holdings.count(Decimal("9")) >= 2, heard.holdings
    assert heard.portfolio == [Decimal("9")]
    assert heard.prints == [Decimal("100")]
    assert heard.quotes == [(Decimal("100"), Decimal("200"))]
    assert heard.depth == [Decimal("5")]
    assert heard.ticks == [Decimal("1")]
    assert heard.buckets == [Decimal("42")]
    assert heard.bars == [(Decimal("100"), Decimal("0")), (Decimal("0.5342"), Decimal("0"))]


def test_a_fresh_record_leaves_its_size_unset():
    """The reference builds each of these with UNSET_DECIMAL in the field."""
    fresh = [
        ibkr_dx.Execution().shares,
        ibkr_dx.Execution().cumQty,
        ibkr_dx.BarData().volume,
        ibkr_dx.BarData().wap,
        ibkr_dx.HistoricalTick().size,
        ibkr_dx.HistoricalTickLast().size,
        ibkr_dx.HistoricalTickBidAsk().sizeBid,
        ibkr_dx.HistoricalTickBidAsk().sizeAsk,
    ]
    for value in fresh:
        assert isinstance(value, Decimal), f"got {type(value).__name__} {value!r}"
        assert value == UNSET_DECIMAL
