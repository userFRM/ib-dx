"""The protobuf-encoding surface the reference client carries, present here.

That client publishes, beside every text request, a `*ProtoBuf` twin that
serializes a request message and sends it on a socket, `useProtoBuf` to ask
which encoding to speak, and a `*ProtoBuf` stub on its wrapper for every
answer of that encoding. This client has no socket between a program and its
engine, so the family routes each message to the text request that carries
the same intent, `useProtoBuf` answers False — a sample program written
against that client takes its text path — and the wrapper stubs exist to be
found by a `super()` call. Issue #193: the surface was absent, and the
official sample died on its first import line, `printProtoSingleLine` from
utils.
"""

import pytest

from ib_dx import EClient, EWrapper, TagValue
from ib_dx.utils import printProtoSingleLine


class Message:
    """Stands in for a message of that encoding: fields by attribute, and
    `HasField` answering what a real message answers about what was written."""

    def __init__(self, **fields):
        self.__dict__.update(fields)

    def HasField(self, name):
        return name in self.__dict__


class Entry(Message):
    """One entry of a map field, which that encoding carries as a message."""


def contract():
    from ib_dx import Contract

    made = Contract()
    made.conId = 756733
    made.secType = "STK"
    made.exchange = "SMART"
    made.currency = "USD"
    made.primaryExchange = "NYSE"
    made.multiplier = "100"
    return made


def contract_message():
    return Message(conId=756733, secType="STK", exchange="SMART", currency="USD",
                   primaryExch="NYSE", multiplier=100.0)


class App(EWrapper):
    def __init__(self):
        super().__init__()
        self.calls = []

    def current_time(self, time):
        self.calls.append(time)


def commands(run):
    """What one connected client sends when `run` is called on it."""
    client = EClient(App())
    client._test_connect("DU1")
    client._test_map_con_id(756733, 0)
    run(client)
    return client._test_take_commands()


#: The client family the reference defines, under its own spellings. The two
#: config calls answer from the Rust surface and are listed with the rest.
CLIENT_FAMILY = (
    "useProtoBuf", "startApiProtoBuf", "reqCurrentTimeProtoBuf",
    "setServerLogLevelProtoBuf", "reqMarketDataProtoBuf",
    "cancelMarketDataProtoBuf", "reqMarketDataTypeProtoBuf",
    "reqSmartComponentsProtoBuf", "reqMarketRuleProtoBuf",
    "reqTickByTickDataProtoBuf", "cancelTickByTickProtoBuf",
    "calculateImpliedVolatilityProtoBuf", "cancelCalculateImpliedVolatilityProtoBuf",
    "calculateOptionPriceProtoBuf", "cancelCalculateOptionPriceProtoBuf",
    "exerciseOptionsProtoBuf", "placeOrderProtoBuf", "cancelOrderProtoBuf",
    "reqOpenOrdersProtoBuf", "reqAutoOpenOrdersProtoBuf", "reqAllOpenOrdersProtoBuf",
    "reqGlobalCancelProtoBuf", "reqIdsProtoBuf", "reqAccountUpdatesProtoBuf",
    "reqAccountSummaryProtoBuf", "cancelAccountSummaryProtoBuf",
    "reqPositionsProtoBuf", "cancelPositionsProtoBuf", "reqPositionsMultiProtoBuf",
    "cancelPositionsMultiProtoBuf", "reqAccountUpdatesMultiProtoBuf",
    "cancelAccountUpdatesMultiProtoBuf", "reqPnLProtoBuf", "cancelPnLProtoBuf",
    "reqPnLSingleProtoBuf", "cancelPnLSingleProtoBuf", "reqExecutionsProtoBuf",
    "reqContractDataProtoBuf", "reqMarketDepthExchangesProtoBuf",
    "reqMarketDepthProtoBuf", "cancelMarketDepthProtoBuf", "reqNewsBulletinsProtoBuf",
    "cancelNewsBulletinsProtoBuf", "reqManagedAcctsProtoBuf", "reqFAProtoBuf",
    "replaceFAProtoBuf", "reqHistoricalDataProtoBuf", "cancelHistoricalDataProtoBuf",
    "reqHeadTimestampProtoBuf", "cancelHeadTimestampProtoBuf",
    "reqHistogramDataProtoBuf", "cancelHistogramDataProtoBuf",
    "reqHistoricalTicksProtoBuf", "reqScannerParametersProtoBuf",
    "reqScannerSubscriptionProtoBuf", "cancelScannerSubscriptionProtoBuf",
    "reqRealTimeBarsProtoBuf", "cancelRealTimeBarsProtoBuf",
    "reqNewsProvidersProtoBuf", "reqNewsArticleProtoBuf", "reqHistoricalNewsProtoBuf",
    "queryDisplayGroupsProtoBuf", "subscribeToGroupEventsProtoBuf",
    "updateDisplayGroupProtoBuf", "unsubscribeFromGroupEventsProtoBuf",
    "verifyRequestProtoBuf", "verifyMessageProtoBuf", "reqSecDefOptParamsProtoBuf",
    "reqSoftDollarTiersProtoBuf", "reqFamilyCodesProtoBuf", "reqMatchingSymbolsProtoBuf",
    "reqCompletedOrdersProtoBuf", "reqWshMetaDataProtoBuf", "cancelWshMetaDataProtoBuf",
    "reqWshEventDataProtoBuf", "cancelWshEventDataProtoBuf", "reqUserInfoProtoBuf",
    "reqCurrentTimeInMillisProtoBuf", "cancelContractDataProtoBuf",
    "cancelHistoricalTicksProtoBuf", "reqConfigProtoBuf", "updateConfigProtoBuf",
)

#: The wrapper stubs the reference defines, under its own spellings.
WRAPPER_STUBS = (
    "orderStatusProtoBuf", "openOrderProtoBuf", "openOrdersEndProtoBuf",
    "errorProtoBuf", "executionDetailsProtoBuf", "executionDetailsEndProtoBuf",
    "completedOrderProtoBuf", "completedOrdersEndProtoBuf", "orderBoundProtoBuf",
    "contractDataProtoBuf", "bondContractDataProtoBuf", "contractDataEndProtoBuf",
    "tickPriceProtoBuf", "tickSizeProtoBuf", "tickOptionComputationProtoBuf",
    "tickGenericProtoBuf", "tickStringProtoBuf", "tickSnapshotEndProtoBuf",
    "updateMarketDepthProtoBuf", "updateMarketDepthL2ProtoBuf",
    "updateMarketDataTypeProtoBuf", "tickReqParamsProtoBuf", "updateAccountValueProtoBuf",
    "updatePortfolioProtoBuf", "updateAccountTimeProtoBuf", "accountDataEndProtoBuf",
    "managedAccountsProtoBuf", "positionProtoBuf", "positionEndProtoBuf",
    "accountSummaryProtoBuf", "accountSummaryEndProtoBuf", "positionMultiProtoBuf",
    "positionMultiEndProtoBuf", "accountUpdateMultiProtoBuf",
    "accountUpdateMultiEndProtoBuf", "historicalDataProtoBuf",
    "historicalDataUpdateProtoBuf", "historicalDataEndProtoBuf",
    "realTimeBarTickProtoBuf", "headTimestampProtoBuf", "histogramDataProtoBuf",
    "historicalTicksProtoBuf", "historicalTicksBidAskProtoBuf",
    "historicalTicksLastProtoBuf", "tickByTickDataProtoBuf", "updateNewsBulletinProtoBuf",
    "newsArticleProtoBuf", "newsProvidersProtoBuf", "historicalNewsProtoBuf",
    "historicalNewsEndProtoBuf", "wshMetaDataProtoBuf", "wshEventDataProtoBuf",
    "tickNewsProtoBuf", "scannerParametersProtoBuf", "scannerDataProtoBuf",
    "pnlProtoBuf", "pnlSingleProtoBuf", "receiveFAProtoBuf", "replaceFAEndProtoBuf",
    "commissionAndFeesReportProtoBuf", "historicalScheduleProtoBuf",
    "rerouteMarketDataRequestProtoBuf", "rerouteMarketDepthRequestProtoBuf",
    "secDefOptParameterProtoBuf", "secDefOptParameterEndProtoBuf",
    "softDollarTiersProtoBuf", "familyCodesProtoBuf", "symbolSamplesProtoBuf",
    "smartComponentsProtoBuf", "marketRuleProtoBuf", "userInfoProtoBuf",
    "nextValidIdProtoBuf", "currentTimeProtoBuf", "currentTimeInMillisProtoBuf",
    "verifyMessageApiProtoBuf", "verifyCompletedProtoBuf", "displayGroupListProtoBuf",
    "displayGroupUpdatedProtoBuf", "marketDepthExchangesProtoBuf",
    "configResponseProtoBuf", "updateConfigResponseProtoBuf",
)


@pytest.mark.parametrize("name", CLIENT_FAMILY)
def test_the_client_family_is_there_under_the_reference_spelling(name):
    assert hasattr(EClient, name)


@pytest.mark.parametrize("name", WRAPPER_STUBS)
def test_the_wrapper_stubs_are_there_under_the_reference_spelling(name):
    assert hasattr(EWrapper, name)


def test_the_encoding_question_is_answered_no_under_both_spellings():
    client = EClient(App())
    client._test_connect("DU1")
    assert client.useProtoBuf(msgId=1) is False
    assert client.use_proto_buf(1) is False
    client.disconnect()


def test_a_message_nobody_stated_sends_nothing():
    sent = commands(lambda c: (c.reqMarketDataProtoBuf(None),
                               c.req_market_data_proto_buf(None)))
    assert sent == []


def test_a_super_call_finds_the_stub_on_the_base():
    class Overriding(App):
        def orderStatusProtoBuf(self, orderStatusProto):
            super().orderStatusProtoBuf(orderStatusProto)
            self.calls.append(orderStatusProto.status)

    app = Overriding()
    app.orderStatusProtoBuf(Message(status="Submitted"))
    assert app.calls == ["Submitted"]


def test_the_display_helper_prints_one_line_under_its_header(capsys):
    class Spread:
        def __str__(self):
            return "status: Submitted\nclientId: 2"

    printProtoSingleLine("OrderStatus.", Spread())
    printed = capsys.readouterr().out
    assert printed.startswith("OrderStatus. ")
    assert len(printed.strip().splitlines()) == 1


#: A call on the second encoding carries the same intent as the text call
#: beside it: the commands the two leave behind are the same commands.
PARITY = (
    ("market data",
     lambda c: c.reqMarketDataProtoBuf(Message(reqId=1, contract=contract_message())),
     lambda c: c.req_mkt_data(1, contract())),
    ("an order with an algo parameter and an oca group",
     lambda c: c.placeOrderProtoBuf(Message(
         orderId=7, contract=contract_message(),
         order=Message(action="BUY", totalQuantity="100", orderType="LMT",
                       lmtPrice=1.5, tif="DAY", ocaGroup="1234",
                       algoParams=[Entry(key="x", value="y")],
                       algoStrategy="Adaptive"))),
     lambda c: c.place_order(7, contract(), _order_text())),
    ("a cancel by its number",
     lambda c: c.cancelPnLProtoBuf(Message(reqId=9)),
     lambda c: c.cancel_pnl(9)),
    ("a cancel stating no number, which the encoding carries as zero",
     lambda c: c.cancelPnLProtoBuf(Message()),
     lambda c: c.cancel_pnl(0)),
    ("historical ticks counting what the message states",
     lambda c: c.reqHistoricalTicksProtoBuf(Message(
         reqId=3, contract=contract_message(), numberOfTicks=7, whatToShow="TRADES")),
     lambda c: c.req_historical_ticks(3, contract(), number_of_ticks=7,
                                      what_to_show="TRADES")),
    ("historical ticks counting what neither states",
     lambda c: c.reqHistoricalTicksProtoBuf(Message(reqId=3, contract=contract_message())),
     lambda c: c.req_historical_ticks(3, contract())),
    ("an execution filter with its window stated",
     lambda c: c.reqExecutionsProtoBuf(Message(
         reqId=4, executionFilter=Message(clientId=1, lastNDays=3))),
     lambda c: c.req_executions(4, _execution_filter_text())),
    ("a watchlist event request, whose fields the message carries flat",
     lambda c: c.reqWshEventDataProtoBuf(Message(reqId=5, conId=756733, totalLimit=10)),
     lambda c: c.req_wsh_event_data(5, _wsh_text())),
    ("a scan with its option pairs",
     lambda c: c.reqScannerSubscriptionProtoBuf(Message(
         reqId=6, scannerSubscription=Message(
             instrument="STK", locationCode="STK.US.MAJOR", scanCode="MOST_ACTIVE",
             scannerSubscriptionOptions=[Entry(key="manual", value="1")],
             scannerSubscriptionFilterOptions=[
                 Entry(key="usdMarketCapAbove", value="1000000")]))),
     lambda c: c.req_scanner_subscription(6, _scanner_text(),
                                          [TagValue("manual", "1")],
                                          [TagValue("usdMarketCapAbove",
                                                    "1000000")])),
    ("a market-depth cancel that states its smart depth",
     lambda c: c.cancelMarketDepthProtoBuf(Message(reqId=8, isSmartDepth=True)),
     lambda c: c.cancel_mkt_depth(8, True)),
)


def _order_text():
    from ib_dx import Order, TagValue

    made = Order()
    made.action = "BUY"
    made.totalQuantity = "100"
    made.orderType = "LMT"
    made.lmtPrice = 1.5
    made.tif = "DAY"
    made.ocaGroup = "1234"
    made.algoParams = [TagValue("x", "y")]
    made.algoStrategy = "Adaptive"
    return made


def _execution_filter_text():
    from ib_dx import ExecutionFilter

    made = ExecutionFilter()
    made.clientId = 1
    made.lastNDays = 3
    return made


def _wsh_text():
    from ib_dx import WshEventData

    made = WshEventData()
    made.conId = 756733
    made.totalLimit = 10
    return made


def _scanner_text():
    from ib_dx import ScannerSubscription

    made = ScannerSubscription()
    made.instrument = "STK"
    made.locationCode = "STK.US.MAJOR"
    made.scanCode = "MOST_ACTIVE"
    return made


@pytest.mark.parametrize("name,proto_call,text_call", PARITY,
                         ids=[row[0] for row in PARITY])
def test_a_call_on_the_second_encoding_sends_what_its_text_twin_sends(
        name, proto_call, text_call):
    assert commands(proto_call) == commands(text_call)


def test_the_sample_style_program_answers_both_current_time_requests():
    app = App()
    client = EClient(app)
    client._test_connect("DU1")
    client.reqCurrentTime()
    client.reqCurrentTimeProtoBuf(Message())
    client.poll()
    assert app.calls, "both current-time requests were answered"
    client.disconnect()
