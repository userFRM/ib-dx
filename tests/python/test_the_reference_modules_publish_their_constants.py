"""The reference client's modules publish their constants and enumerations here too.

A program star-imports `ibkr_dx.order` and writes `order.origin = CUSTOMER`, as
that client's own `Order.__init__` does; it compares a scan's row count with
`NO_ROW_NUMBER_SPECIFIED`, a fill's exercise type with
`OptionExerciseType.NoneItem`, a fund's class with `FundAssetType.Equity`.
None of those names existed here, so each such line was a NameError or, for
the enumerations, a comparison that could never hold.

Run: pytest tests/python/test_the_reference_modules_publish_their_constants.py -v
"""

from ibkr_dx import contract, execution, news, order, scanner
from ibkr_dx import ContractDetails, Execution, Order, ScannerSubscription


def test_the_order_module_publishes_the_origin_and_auction_constants():
    assert (order.CUSTOMER, order.FIRM, order.UNKNOWN) == (0, 1, 2)
    assert (order.AUCTION_UNSET, order.AUCTION_MATCH, order.AUCTION_IMPROVEMENT, order.AUCTION_TRANSPARENT) == (0, 1, 2, 3)
    made = Order()
    assert made.origin == order.CUSTOMER
    assert made.auctionStrategy == order.AUCTION_UNSET


def test_the_contract_module_publishes_the_leg_positions():
    assert (contract.SAME_POS, contract.OPEN_POS, contract.CLOSE_POS, contract.UNKNOWN_POS) == (0, 1, 2, 3)


def test_the_scanner_and_news_modules_publish_their_constants():
    assert scanner.NO_ROW_NUMBER_SPECIFIED == -1
    assert ScannerSubscription().numberOfRows == scanner.NO_ROW_NUMBER_SPECIFIED
    assert (news.NEWS_MSG, news.EXCHANGE_AVAIL_MSG, news.EXCHANGE_UNAVAIL_MSG) == (1, 2, 3)


def test_an_exercise_type_is_the_enumeration_member():
    kinds = execution.OptionExerciseType
    fill = Execution()
    assert fill.optExerciseOrLapseType is kinds.NoneItem
    fill.optExerciseOrLapseType = kinds.Lapse
    assert fill.optExerciseOrLapseType is kinds.Lapse
    fill.opt_exercise_or_lapse_type = 100
    assert fill.optExerciseOrLapseType is kinds.Assigned
    fill.opt_exercise_or_lapse_type = 12345
    assert fill.optExerciseOrLapseType is kinds.NoneItem, "an unlisted code reads as the first member, as it does there"


def test_a_funds_class_and_policy_are_the_enumeration_members():
    details = ContractDetails()
    assert details.fundAssetType is contract.FundAssetType.NoneItem
    assert details.fundDistributionPolicyIndicator is contract.FundDistributionPolicyIndicator.NoneItem
    details.fundAssetType = contract.FundAssetType.Equity
    assert details.fundAssetType is contract.FundAssetType.Equity
    details.fund_asset_type = "001"
    assert details.fundAssetType is contract.FundAssetType.MoneyMarket
    details.fundDistributionPolicyIndicator = "Y"
    assert details.fundDistributionPolicyIndicator is contract.FundDistributionPolicyIndicator.IncomeFund


def test_the_errors_module_publishes_the_numbers_this_client_reports_itself():
    # A program compares a callback's number against these rather than a bare
    # 501, so the codes and the standing words are the contract.
    from ibkr_dx import errors

    assert (errors.ALREADY_CONNECTED.errorCode, errors.ALREADY_CONNECTED.errorMsg) == (501, "Already connected.")
    assert (errors.NOT_CONNECTED.errorCode, errors.NOT_CONNECTED.errorMsg) == (504, "Not connected")
    assert (errors.BAD_MESSAGE.errorCode, errors.BAD_MESSAGE.errorMsg) == (508, "Bad message")


def test_the_order_status_module_publishes_the_states_a_callback_carries():
    from ibkr_dx.order_status import OrderStatus

    assert str(OrderStatus.Filled) == "Filled"
    assert OrderStatus.get("filled") is OrderStatus.Filled
    assert OrderStatus.get(" PendingSubmit ") is OrderStatus.PendingSubmit
    assert OrderStatus.get("") is OrderStatus.Unknown
    assert OrderStatus.get("no such state") is OrderStatus.Unknown
    assert OrderStatus.Submitted.is_active() and not OrderStatus.Submitted.is_terminal()
    assert OrderStatus.Filled.is_terminal() and not OrderStatus.Filled.is_active()


def test_the_utils_module_publishes_its_helpers():
    import time

    from ibkr_dx import utils
    from ibkr_dx.execution import OptionExerciseType

    assert utils.isValidFloatValue(1.0) and not utils.isValidFloatValue(utils.UNSET_DOUBLE)
    assert utils.isAsciiPrintable("host\tname") and not utils.isAsciiPrintable("hé")
    assert utils.isPegBenchOrder("PEGBENCH") and utils.isPegBenchOrder("PEG BENCH")
    assert utils.isPegMidOrder("PEG MID") and not utils.isPegMidOrder("PEGBENCH")
    assert utils.isPegBestOrder("PEG BEST")
    assert abs(utils.currentTimeMillis() - time.time() * 1000) < 5000
    assert utils.listOfValues(OptionExerciseType)[0] is OptionExerciseType.NoneItem
    assert utils.getEnumTypeFromString(OptionExerciseType, 2) is OptionExerciseType.Lapse
    assert utils.getEnumTypeFromString(OptionExerciseType, "no such") is OptionExerciseType.NoneItem


def test_the_common_module_publishes_the_plain_enum_holder():
    from ibkr_dx import common

    assert common.MarketDataType is int
    assert common.FaDataType is int
    assert common.Liquidities is int
    assert common.LiquiditiesEnum.Added == 1
    # The last name is spelled as the reference client spells it.
    assert common.LiquiditiesEnum.toStr(3) == "RoudedOut"
    assert common.LiquiditiesEnum.toStr(9) == "NOTFOUND"
    made = common.Enum("a", "b")
    assert made.b == 1 and made.toStr(0) == "a"
