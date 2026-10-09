"""A record prints the sentence the reference client writes.

Printing a record here once answered an invented field list — and an unset
Order double printed as its 309-digit sentinel — instead of the reference
client's own sentence, built through the max-string helpers so a value
nobody set prints blank; Execution printed the generic object repr at all.
A log scraper or a test matching those sentences found nothing it
recognises. Every record now carries the reference's own algorithm and the
Object base's repr — the id, a colon and that same text. The expected
strings are what the reference client's own classes print for the same
field values.

Run: pytest tests/python/test_a_record_prints_the_text_the_reference_writes.py -v
"""

import re
from decimal import Decimal

import ib_dx


def _idless(text):
    # A list of records renders each through the Object repr, which leads
    # with the object's id — different on every run, in the reference too.
    return re.sub(r"\d{12,}(?=: )", "ID", text)


def _prints(record, expected):
    assert _idless(str(record)) == _idless(expected)
    assert _idless(repr(record)) == _idless(f"{id(record)}: {expected}")


def test_fresh_records_print_the_reference_sentences():
    _prints(ib_dx.Order(), "0,0,0:   @ ")
    _prints(ib_dx.OrderComboLeg(), "")
    _prints(ib_dx.OrderAllocation(),
            "Account: , Position: , PositionDesired: , PositionAfter: , "
            "DesiredAllocQty: , AllowedAllocQty: , IsMonetary: False")
    _prints(ib_dx.Contract(),
            "ConId: 0, Symbol: , SecType: , LastTradeDateOrContractMonth: , "
            "Strike: , Right: , Multiplier: , Exchange: , PrimaryExchange: , "
            "Currency: , LocalSymbol: , TradingClass: , IncludeExpired: False, "
            "SecIdType: , SecId: , Description: , IssuerId: Combo:")
    _prints(ib_dx.ComboLeg(), "0,0,,,0,0,,-1")
    _prints(ib_dx.DeltaNeutralContract(), "0,0,0")
    _prints(ib_dx.Execution(),
            "ExecId: , Time: , Account: , Exchange: , Side: , Shares: , "
            "Price: 0, PermId: 0, ClientId: 0, OrderId: 0, Liquidation: 0, "
            "CumQty: , AvgPrice: 0, OrderRef: , EvRule: , EvMultiplier: 0, "
            "ModelCode: , LastLiquidity: 0, PendingPriceRevision: False, "
            "Submitter: , OptExerciseOrLapseType: None")
    _prints(ib_dx.BarData(),
            "Date: , Open: 0, High: 0, Low: 0, Close: 0, Volume: , WAP: , BarCount: 0")
    # The one place a fresh record reads differently from a fresh reference
    # record: its constructor births the coupon as the integer 0 while its
    # decoder — like this client's — keeps a float, and a details off the
    # wire reads the same under both. The field here is that float, so a
    # built one prints 0.0 where the reference's prints 0.
    _prints(ib_dx.ContractDetails(),
            "ConId: 0, Symbol: , SecType: , LastTradeDateOrContractMonth: , Strike: , "
            "Right: , Multiplier: , Exchange: , PrimaryExchange: , Currency: , "
            "LocalSymbol: , TradingClass: , IncludeExpired: False, SecIdType: , "
            "SecId: , Description: , IssuerId: Combo:,,0,,,0,0,,,,,,,,,,0,,,,0,"
            "None,,,,,,,,False,False,0.0,False,,,,,False,,,,,,,,None,,,,")
    _prints(ib_dx.OrderState(),
            "Status: , InitMarginBefore: , MaintMarginBefore: , EquityWithLoanBefore: , "
            "InitMarginChange: , MaintMarginChange: , EquityWithLoanChange: , "
            "InitMarginAfter: , MaintMarginAfter: , EquityWithLoanAfter: , "
            "CommissionAndFees: , MinCommissionAndFees: , MaxCommissionAndFees: , "
            "CommissionAndFeesCurrency: , MarginCurrency: , "
            "InitMarginBeforeOutsideRTH: , MaintMarginBeforeOutsideRTH: , EquityWithLoanBeforeOutsideRTH: , "
            "InitMarginChangeOutsideRTH: , MaintMarginChangeOutsideRTH: , equityWithLoanChangeOutsideRTH: , "
            "InitMarginAfterOutsideRTH: , MaintMarginAfterOutsideRTH: , equityWithLoanAfterOutsideRTH: , "
            "SuggestedSize: , RejectReason: , WarningText: , CompletedTime: , CompletedStatus: ")


def test_populated_records_print_the_reference_sentences():
    contract = ib_dx.Contract()
    contract.conId = 1234
    contract.symbol = "AAPL"
    contract.secType = "OPT"
    contract.lastTradeDateOrContractMonth = "20260918"
    contract.strike = 200.0
    contract.right = "C"
    contract.multiplier = "100"
    contract.exchange = "SMART"
    contract.primaryExchange = "NASDAQ"
    contract.currency = "USD"
    contract.localSymbol = "AAPL  260918C00200000"
    contract.tradingClass = "AAPL"
    contract.includeExpired = True
    contract.secIdType = "ISIN"
    contract.secId = "US0378331005"
    contract.description = "Apple Inc"
    contract.issuerId = "E1234"
    contract.comboLegsDescrip = "AAPL"
    leg = ib_dx.ComboLeg()
    leg.conId = 756733
    leg.ratio = 1
    leg.action = "BUY"
    leg.exchange = "SMART"
    leg.openClose = 1
    leg.shortSaleSlot = 0
    leg.designatedLocation = ""
    leg.exemptCode = -1
    contract.comboLegs = [leg]
    hedge = ib_dx.DeltaNeutralContract()
    hedge.conId = 756733
    hedge.delta = 0.5
    hedge.price = 200.0
    contract.deltaNeutralContract = hedge
    _prints(contract,
            "ConId: 1234, Symbol: AAPL, SecType: OPT, LastTradeDateOrContractMonth: 20260918, "
            "Strike: 200, Right: C, Multiplier: 100, Exchange: SMART, PrimaryExchange: NASDAQ, "
            "Currency: USD, LocalSymbol: AAPL  260918C00200000, TradingClass: AAPL, "
            "IncludeExpired: True, SecIdType: ISIN, SecId: US0378331005, Description: Apple Inc, "
            "IssuerId: E1234Combo:AAPL;756733,1,BUY,SMART,1,0,,-1;756733,0.5,200")

    order = ib_dx.Order()
    order.orderId = 7
    order.clientId = 1
    order.permId = 123456789
    order.orderType = "LMT"
    order.action = "BUY"
    order.totalQuantity = Decimal("100")
    order.lmtPrice = 200.5
    order.tif = "DAY"
    combo_leg = ib_dx.OrderComboLeg()
    combo_leg.price = 1.5
    order.orderComboLegs = [combo_leg]
    order.conditions = [ib_dx.PriceCondition(0, 756733, "SMART", True, 200.0)]
    _prints(order,
            "7,1,123456789: LMT BUY 100@200.5 DAY CMB(1.5,) "
            "COND(Default price of 756733 on SMART is  >=  200.0  ,)")

    state = ib_dx.OrderState()
    state.status = "Submitted"
    state.initMarginBefore = "100.0"
    state.commissionAndFees = 1.25
    state.minCommissionAndFees = 1.0
    state.maxCommissionAndFees = 2.0
    state.commissionAndFeesCurrency = "USD"
    state.marginCurrency = "USD"
    state.suggestedSize = Decimal("100")
    state.completedTime = "20260930 10:00:00"
    state.completedStatus = "Filled"
    allocation = ib_dx.OrderAllocation()
    allocation.account = "DU1"
    allocation.position = Decimal("100")
    allocation.positionDesired = Decimal("200")
    allocation.positionAfter = Decimal("200")
    allocation.desiredAllocQty = Decimal("100")
    allocation.allowedAllocQty = Decimal("100")
    allocation.isMonetary = True
    state.orderAllocations = [allocation]
    _prints(state,
            "Status: Submitted, InitMarginBefore: 100.0, MaintMarginBefore: , "
            "EquityWithLoanBefore: , InitMarginChange: , MaintMarginChange: , "
            "EquityWithLoanChange: , InitMarginAfter: , MaintMarginAfter: , "
            "EquityWithLoanAfter: , CommissionAndFees: 1.25, MinCommissionAndFees: 1, "
            "MaxCommissionAndFees: 2, CommissionAndFeesCurrency: USD, MarginCurrency: USD, "
            "InitMarginBeforeOutsideRTH: , MaintMarginBeforeOutsideRTH: , EquityWithLoanBeforeOutsideRTH: , "
            "InitMarginChangeOutsideRTH: , MaintMarginChangeOutsideRTH: , equityWithLoanChangeOutsideRTH: , "
            "InitMarginAfterOutsideRTH: , MaintMarginAfterOutsideRTH: , equityWithLoanAfterOutsideRTH: , "
            "SuggestedSize: 100, RejectReason: , WarningText: , "
            "CompletedTime: 20260930 10:00:00, CompletedStatus: Filled "
            "OrderAllocations(Account: DU1, Position: 100, PositionDesired: 200, "
            "PositionAfter: 200, DesiredAllocQty: 100, AllowedAllocQty: 100, "
            "IsMonetary: True; )")

    execution = ib_dx.Execution()
    execution.execId = "0001"
    execution.time = "20260930  10:00:00"
    execution.acctNumber = "DU1"
    execution.exchange = "NASDAQ"
    execution.side = "BOT"
    execution.shares = Decimal("100")
    execution.price = 200.5
    execution.permId = 123456789
    execution.clientId = 1
    execution.orderId = 7
    execution.liquidation = 0
    execution.cumQty = Decimal("100")
    execution.avgPrice = 200.5
    execution.orderRef = "1"
    execution.evRule = "EV"
    execution.evMultiplier = 25.0
    execution.modelCode = "MC"
    execution.lastLiquidity = 1
    execution.pendingPriceRevision = False
    execution.submitter = "SUB"
    execution.optExerciseOrLapseType = -1
    _prints(execution,
            "ExecId: 0001, Time: 20260930  10:00:00, Account: DU1, Exchange: NASDAQ, "
            "Side: BOT, Shares: 100, Price: 200.5, PermId: 123456789, ClientId: 1, "
            "OrderId: 7, Liquidation: 0, CumQty: 100, AvgPrice: 200.5, OrderRef: 1, "
            "EvRule: EV, EvMultiplier: 25, ModelCode: MC, LastLiquidity: 1, "
            "PendingPriceRevision: False, Submitter: SUB, OptExerciseOrLapseType: None")

    bar = ib_dx.BarData("20260930", 1.0, 2.0, 0.5, 1.5,
                          Decimal("1000"), Decimal("1.25"), 3)
    _prints(bar,
            "Date: 20260930, Open: 1, High: 2, Low: 0.5, Close: 1.5, "
            "Volume: 1000, WAP: 1.25, BarCount: 3")


def test_details_print_their_lists_as_the_reference_renders_them():
    details = ib_dx.ContractDetails()
    details.marketName = "AAPL"
    details.minTick = 0.01
    details.evMultiplier = 25
    details.coupon = 5.5
    details.putable = True
    details.notes = "7"
    details.minSize = Decimal("1")
    details.lastPricePrecision = Decimal("0.010")
    details.secIdList = [ib_dx.TagValue("ISIN", "US0378331005")]
    assert ",AAPL,0.01," in str(details)
    assert ",5.5,False," in str(details), "the coupon reads as the float it is"
    assert ",False,True," in str(details), "the bond flags read under their reference names"
    assert ",0.010," in str(details), "a precision keeps the digits it was stated with"
    assert "[ID: ISIN=US0378331005;]" in _idless(str(details))

    tag = ib_dx.TagValue("ISIN", "US0378331005")
    _prints(tag, "ISIN=US0378331005;")
    reason = ib_dx.IneligibilityReason("1", "no borrowing")
    _prints(reason, "[id: 1, description: no borrowing];")
