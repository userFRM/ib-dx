"""None on a data class is stored, and the send refuses it.

The reference client's data classes are plain Python: every attribute takes
None, construction never refuses it, and the failure comes only at send time,
where the encoder raises on None and the sending function says it on the error
callback under that request's own sending-error code — for `placeOrder`, 512,
"Order Sending Error - Cannot send None to TWS", under the order's id.
"""

from ibkr_dx import Contract, EClient, EWrapper, Order


class _Recorder(EWrapper):
    def __init__(self):
        super().__init__()
        self.errors = []
        self.placed = {}

    def error(self, req_id, error_time, code, msg, advanced=""):
        self.errors.append((req_id, code, msg))

    def open_order(self, order_id, contract, order, order_state):
        self.placed[order_id] = (contract, order)


SPY_CON_ID = 756733


def _session():
    recorder = _Recorder()
    client = EClient(recorder)
    client._test_connect("DU0000000")
    client._test_map_con_id(SPY_CON_ID, 0)
    return recorder, client


def _spy():
    contract = Contract()
    contract.symbol, contract.secType, contract.exchange, contract.currency = "SPY", "STK", "SMART", "USD"
    contract.conId = SPY_CON_ID
    return contract


def _limit(action, quantity, price):
    order = Order()
    order.action = action
    order.orderType = "LMT"
    order.totalQuantity = quantity
    order.lmtPrice = price
    return order


def test_none_is_stored_and_read_back_under_either_spelling():
    order = Order(account=None)
    assert order.account is None
    order.account = "U123"
    assert order.account == "U123"

    order.lmtPrice = None
    assert order.lmt_price is None
    order.lmt_price = 10.0
    assert order.lmtPrice == 10.0

    contract = Contract(symbol=None)
    assert contract.symbol is None
    assert contract.secType == ""


def test_a_send_carrying_none_is_refused_under_the_sending_error():
    recorder, client = _session()
    order = _limit("BUY", 1, 10.0)
    order.account = None
    client.place_order(1001, _spy(), order)
    client.poll()
    assert recorder.errors == [
        (1001, 512, "Order Sending Error - Cannot send None to TWS")
    ], recorder.errors
    assert not recorder.placed, "nothing may go to the venue"

    contract = _spy()
    contract.symbol = None
    client.place_order(1002, contract, _limit("BUY", 1, 10.0))
    client.poll()
    assert recorder.errors[-1] == (
        1002, 512, "Order Sending Error - Cannot send None to TWS"
    ), recorder.errors
    assert not recorder.placed, "nothing may go to the venue"
