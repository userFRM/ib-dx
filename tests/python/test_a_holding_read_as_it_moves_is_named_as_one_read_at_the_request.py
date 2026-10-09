"""A holding read as it moves is named as one read at the request.

The position row the venue states carries the symbol, the security type,
the currency and the multiplier, and the positions path already builds the
contract it delivers from those fields while the definition cache is cold.
The account-subscription path dropped them: its row kept only the con id
and the figures, so a holding delivered on updatePortfolio before its
definition landed arrived nameless — and stayed nameless until a figure
moved, since a quiet row is not re-emitted. A caller keying its ledger by
the contract's symbol read "".
"""

import time

import ib_dx


class Heard(ib_dx.EWrapper):
    def __init__(self):
        super().__init__()
        self.portfolio = []
        self.positions = []

    def updatePortfolio(self, contract, pos, marketPrice, marketValue,
                        averageCost, unrealizedPNL, realizedPNL, accountName):
        self.portfolio.append(contract)

    def position(self, account, contract, pos, avgCost):
        self.positions.append(contract)


def _names(contract):
    return (contract.symbol, contract.secType, contract.currency,
            contract.multiplier)


def test_the_streamed_holding_carries_the_names_the_row_states():
    heard = Heard()
    client = ib_dx.EClient(heard)
    client._test_connect("DU1", True, accounts=["DU1"])
    client._test_set_position(756733, 9, 30, account="DU1", symbol="AAPL",
                              sec_type="STK", currency="USD", multiplier="1")
    client._test_set_account(net_liquidation=100, daily_pnl=1, account="DU1")
    client._test_finish_account_download("DU1")
    client.reqPositions()
    client.reqAccountUpdates(True, "DU1")
    client.poll()

    deadline = time.monotonic() + 5
    while (not heard.portfolio or not heard.positions) and time.monotonic() < deadline:
        client._test_dispatch_once()
        time.sleep(0.05)

    assert heard.positions, "the holding read at the request reached nothing"
    assert heard.portfolio, "the holding read as it moves reached nothing"
    # Named from the row, exactly as the positions path names it.
    assert _names(heard.portfolio[0]) == ("AAPL", "STK", "USD", "1")
    assert _names(heard.portfolio[0]) == _names(heard.positions[0])
