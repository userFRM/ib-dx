"""The sentences the reference client's record classes print, laid over ours.

The classes here are the engine's own; the reference client writes each
record as a sentence of its own — built from the max-string helpers, so a
value nobody set prints blank rather than as this client's unset markers
(ibapi contract.py, order.py, order_state.py, execution.py, common.py). A
program's log scraper reads those sentences, so each class here carries the
reference's own algorithm for it, and repr is the Object base's — the id, a
colon and that same text.
"""

from ._reference_shapes import (
    OptionExerciseType,
    decimalMaxString,
    floatMaxString,
    getEnumTypeName,
    intMaxString,
    longMaxString,
)


def _combo_leg_str(self):
    return ",".join(
        (
            intMaxString(self.conId),
            intMaxString(self.ratio),
            str(self.action),
            str(self.exchange),
            intMaxString(self.openClose),
            intMaxString(self.shortSaleSlot),
            str(self.designatedLocation),
            intMaxString(self.exemptCode),
        )
    )


def _delta_neutral_str(self):
    return ",".join(
        (str(self.conId), floatMaxString(self.delta), floatMaxString(self.price))
    )


def _contract_str(self):
    s = (
        "ConId: %s, Symbol: %s, SecType: %s, LastTradeDateOrContractMonth: %s, Strike: %s, Right: %s, Multiplier: %s, Exchange: %s, PrimaryExchange: %s, "
        "Currency: %s, LocalSymbol: %s, TradingClass: %s, IncludeExpired: %s, SecIdType: %s, SecId: %s, Description: %s, "
        "IssuerId: %s"
        % (
            intMaxString(self.conId),
            str(self.symbol),
            str(self.secType),
            str(self.lastTradeDateOrContractMonth),
            floatMaxString(self.strike),
            str(self.right),
            str(self.multiplier),
            str(self.exchange),
            str(self.primaryExchange),
            str(self.currency),
            str(self.localSymbol),
            str(self.tradingClass),
            str(self.includeExpired),
            str(self.secIdType),
            str(self.secId),
            str(self.description),
            str(self.issuerId),
        )
    )
    s += "Combo:" + self.comboLegsDescrip

    if self.comboLegs:
        for leg in self.comboLegs:
            s += ";" + str(leg)

    if self.deltaNeutralContract:
        s += ";" + str(self.deltaNeutralContract)

    return s


def _contract_details_str(self):
    return ",".join(
        (
            str(self.contract),
            str(self.marketName),
            floatMaxString(self.minTick),
            str(self.orderTypes),
            str(self.validExchanges),
            intMaxString(self.priceMagnifier),
            intMaxString(self.underConId),
            str(self.longName),
            str(self.contractMonth),
            str(self.industry),
            str(self.category),
            str(self.subcategory),
            str(self.timeZoneId),
            str(self.tradingHours),
            str(self.liquidHours),
            str(self.evRule),
            intMaxString(self.evMultiplier),
            str(self.underSymbol),
            str(self.underSecType),
            str(self.marketRuleIds),
            intMaxString(self.aggGroup),
            str(self.secIdList),
            str(self.realExpirationDate),
            str(self.stockType),
            str(self.cusip),
            str(self.ratings),
            str(self.descAppend),
            str(self.bondType),
            str(self.couponType),
            str(self.callable),
            str(self.putable),
            str(self.coupon),
            str(self.convertible),
            str(self.maturity),
            str(self.issueDate),
            str(self.nextOptionDate),
            str(self.nextOptionType),
            str(self.nextOptionPartial),
            str(self.notes),
            decimalMaxString(self.minSize),
            decimalMaxString(self.sizeIncrement),
            decimalMaxString(self.suggestedSizeIncrement),
            decimalMaxString(self.minAlgoSize),
            decimalMaxString(self.lastPricePrecision),
            decimalMaxString(self.lastSizePrecision),
            str(self.ineligibilityReasonList),
            str(self.eventContract1),
            str(self.eventContractDescription1),
            str(self.eventContractDescription2),
            str(self.settlementMethod),
        )
    )


def _order_combo_leg_str(self):
    return f"{floatMaxString(self.price)}"


def _order_str(self):
    s = "%s,%s,%s:" % (
        intMaxString(self.orderId),
        intMaxString(self.clientId),
        longMaxString(self.permId),
    )

    s += " %s %s %s@%s" % (
        self.orderType,
        self.action,
        decimalMaxString(self.totalQuantity),
        floatMaxString(self.lmtPrice),
    )

    s += f" {self.tif}"

    if self.orderComboLegs:
        s += " CMB("
        for leg in self.orderComboLegs:
            s += str(leg) + ","
        s += ")"

    if self.conditions:
        s += " COND("
        for cond in self.conditions:
            s += str(cond) + ","
        s += ")"

    return s


def _order_allocation_str(self):
    return ("Account: %s, Position: %s, PositionDesired: %s, PositionAfter: %s, "
            "DesiredAllocQty: %s, AllowedAllocQty: %s, IsMonetary: %s") % (
        str(self.account),
        decimalMaxString(self.position),
        decimalMaxString(self.positionDesired),
        decimalMaxString(self.positionAfter),
        decimalMaxString(self.desiredAllocQty),
        decimalMaxString(self.allowedAllocQty),
        str(self.isMonetary),
    )


def _order_state_str(self):
    s = ("Status: %s, InitMarginBefore: %s, MaintMarginBefore: %s, EquityWithLoanBefore: %s, "
         "InitMarginChange: %s, MaintMarginChange: %s, EquityWithLoanChange: %s, "
         "InitMarginAfter: %s, MaintMarginAfter: %s, EquityWithLoanAfter: %s, "
         "CommissionAndFees: %s, MinCommissionAndFees: %s, MaxCommissionAndFees: %s, CommissionAndFeesCurrency: %s, MarginCurrency: %s, "
         "InitMarginBeforeOutsideRTH: %s, MaintMarginBeforeOutsideRTH: %s, EquityWithLoanBeforeOutsideRTH: %s, "
         "InitMarginChangeOutsideRTH: %s, MaintMarginChangeOutsideRTH: %s, equityWithLoanChangeOutsideRTH: %s, "
         "InitMarginAfterOutsideRTH: %s, MaintMarginAfterOutsideRTH: %s, equityWithLoanAfterOutsideRTH: %s, "
         "SuggestedSize: %s, RejectReason: %s, WarningText: %s, CompletedTime: %s, CompletedStatus: %s") % (
        str(self.status),
        str(self.initMarginBefore),
        str(self.maintMarginBefore),
        str(self.equityWithLoanBefore),
        str(self.initMarginChange),
        str(self.maintMarginChange),
        str(self.equityWithLoanChange),
        str(self.initMarginAfter),
        str(self.maintMarginAfter),
        str(self.equityWithLoanAfter),
        floatMaxString(self.commissionAndFees),
        floatMaxString(self.minCommissionAndFees),
        floatMaxString(self.maxCommissionAndFees),
        str(self.commissionAndFeesCurrency),
        str(self.marginCurrency),
        floatMaxString(self.initMarginBeforeOutsideRTH),
        floatMaxString(self.maintMarginBeforeOutsideRTH),
        floatMaxString(self.equityWithLoanBeforeOutsideRTH),
        floatMaxString(self.initMarginChangeOutsideRTH),
        floatMaxString(self.maintMarginChangeOutsideRTH),
        floatMaxString(self.equityWithLoanChangeOutsideRTH),
        floatMaxString(self.initMarginAfterOutsideRTH),
        floatMaxString(self.maintMarginAfterOutsideRTH),
        floatMaxString(self.equityWithLoanAfterOutsideRTH),
        decimalMaxString(self.suggestedSize),
        str(self.rejectReason),
        str(self.warningText),
        str(self.completedTime),
        str(self.completedStatus),
    )

    if self.orderAllocations:
        s += " OrderAllocations("
        for orderAllocation in self.orderAllocations:
            s += str(orderAllocation) + "; "
        s += ")"

    return s


def _execution_str(self):
    return (
        "ExecId: %s, Time: %s, Account: %s, Exchange: %s, Side: %s, Shares: %s, Price: %s, PermId: %s, "
        "ClientId: %s, OrderId: %s, Liquidation: %s, CumQty: %s, AvgPrice: %s, OrderRef: %s, EvRule: %s, "
        "EvMultiplier: %s, ModelCode: %s, LastLiquidity: %s, PendingPriceRevision: %s, Submitter: %s, OptExerciseOrLapseType: %s"
        % (
            self.execId,
            self.time,
            self.acctNumber,
            self.exchange,
            self.side,
            decimalMaxString(self.shares),
            floatMaxString(self.price),
            longMaxString(self.permId),
            intMaxString(self.clientId),
            intMaxString(self.orderId),
            intMaxString(self.liquidation),
            decimalMaxString(self.cumQty),
            floatMaxString(self.avgPrice),
            self.orderRef,
            self.evRule,
            floatMaxString(self.evMultiplier),
            self.modelCode,
            intMaxString(self.lastLiquidity),
            self.pendingPriceRevision,
            self.submitter,
            getEnumTypeName(OptionExerciseType, self.optExerciseOrLapseType),
        )
    )


def _tag_value_str(self):
    return f"{self.tag}={self.value};"


def _ineligibility_reason_str(self):
    return f"[id: {self.id_}, description: {self.description}];"


def _bar_data_str(self):
    return (
        f"Date: {self.date}, "
        f"Open: {floatMaxString(self.open)}, "
        f"High: {floatMaxString(self.high)}, "
        f"Low: {floatMaxString(self.low)}, "
        f"Close: {floatMaxString(self.close)}, "
        f"Volume: {decimalMaxString(self.volume)}, "
        f"WAP: {decimalMaxString(self.wap)}, "
        f"BarCount: {intMaxString(self.barCount)}"
    )


#: What each class prints, in the reference client's own words.
_STRINGS = {
    "Contract": _contract_str,
    "ComboLeg": _combo_leg_str,
    "DeltaNeutralContract": _delta_neutral_str,
    "ContractDetails": _contract_details_str,
    "Order": _order_str,
    "OrderComboLeg": _order_combo_leg_str,
    "OrderAllocation": _order_allocation_str,
    "OrderState": _order_state_str,
    "Execution": _execution_str,
    "BarData": _bar_data_str,
    "TagValue": _tag_value_str,
    "IneligibilityReason": _ineligibility_reason_str,
}


def install(surface) -> None:
    """Give each record class the sentence the reference client writes."""
    object_repr = surface["Object"].__repr__
    for name, writes in _STRINGS.items():
        cls = surface[name]
        cls.__str__ = writes
        cls.__repr__ = object_repr
