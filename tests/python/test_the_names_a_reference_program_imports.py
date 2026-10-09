"""A program written against the reference client imports these on line one.

Its samples fill in a plain object and hand it back — an execution filter, a
scanner subscription, a combination leg — name the constant an unset field
carries, and annotate every callback with an alias. All of that is evaluated
before the program does anything, so a name absent here is an ImportError or a
NameError at class-definition time and nothing runs at all.

These are read by attribute on the way to the venue, so the shape is the whole
contract: an object of any type carrying the same attribute names is already
accepted, and these are what a caller reaches for when they have no reason to
write their own.
"""

from decimal import Decimal

import ib_dx


def test_the_objects_a_caller_fills_in_exist_and_are_read():
    execution_filter = ib_dx.ExecutionFilter()
    execution_filter.acctCode = "DU1"
    execution_filter.side = "BUY"
    assert execution_filter.clientId == 0
    assert execution_filter.symbol == ""

    scan = ib_dx.ScannerSubscription()
    scan.instrument = "STK"
    scan.locationCode = "STK.US.MAJOR"
    scan.scanCode = "TOP_PERC_GAIN"
    # An unset bound is the largest number there is, which is what tells this
    # client not to send it. Sent, it would empty the scan rather than widen it.
    assert scan.abovePrice == ib_dx.UNSET_DOUBLE
    assert scan.aboveVolume == ib_dx.UNSET_INTEGER
    assert scan.numberOfRows == -1

    leg = ib_dx.ComboLeg()
    leg.conId = 265598
    leg.ratio = 1
    leg.action = "BUY"
    leg.exchange = "SMART"
    assert leg.exemptCode == -1, "the reference leaves it at minus one, not nought"

    hedge = ib_dx.DeltaNeutralContract()
    hedge.conId = 756733
    hedge.delta = 0.5
    hedge.price = 100.0

    ib_dx.OrderComboLeg()
    ib_dx.OrderCancel()
    ib_dx.WshEventData()


def test_a_filter_a_caller_filled_in_reaches_the_request():
    # The point of the object: this client reads it by attribute, so what the
    # caller set is what the request asks for.
    client = ib_dx.EClient(ib_dx.EWrapper())
    execution_filter = ib_dx.ExecutionFilter()
    execution_filter.symbol = "SPY"
    execution_filter.side = "BUY"
    client.reqExecutions(1, execution_filter)

    scan = ib_dx.ScannerSubscription()
    scan.instrument = "STK"
    scan.locationCode = "STK.US.MAJOR"
    scan.scanCode = "TOP_PERC_GAIN"
    client.reqScannerSubscription(2, scan, [], [])


def test_the_constants_an_unset_field_carries():
    assert ib_dx.UNSET_INTEGER == 2**31 - 1
    assert ib_dx.UNSET_LONG == 2**63 - 1
    assert ib_dx.UNSET_DOUBLE == __import__("sys").float_info.max
    assert str(ib_dx.UNSET_DECIMAL) == str(2**127 - 1)
    assert ib_dx.DOUBLE_INFINITY == float("inf")
    assert ib_dx.INFINITY_STR == "Infinity"
    assert ib_dx.NO_VALID_ID == -1
    assert ib_dx.MAX_MSG_LEN == 0xFFFFFF


def test_the_aliases_a_callback_annotation_names():
    # Evaluated when the class body runs, so a program with annotated
    # overrides — which the reference's own sample has — needs them present
    # before it has done anything.
    assert (ib_dx.TickerId, ib_dx.OrderId, ib_dx.TickType) == (int, int, int)
    assert ib_dx.TagValueList is list
    assert ib_dx.SetOfString is set and ib_dx.SetOfFloat is set
    assert ib_dx.SmartComponentMap is dict
    assert ib_dx.ListOfContractDescription is list
    assert ib_dx.ListOfOrder is list
    assert ib_dx.HistogramDataList is list


def test_the_marker_every_override_carries():
    # It marks and does nothing else; a program will not import without it.
    @ib_dx.iswrapper
    def answered(a, b):
        return a + b

    assert answered(1, 2) == 3


def test_a_figure_is_written_for_a_person_and_an_unset_one_is_not():
    assert ib_dx.intMaxString(7) == "7"
    assert ib_dx.intMaxString(ib_dx.UNSET_INTEGER) == ""
    assert ib_dx.floatMaxString(1.5) == "1.5"
    assert ib_dx.floatMaxString(ib_dx.UNSET_DOUBLE) == ""
    assert ib_dx.longMaxString(ib_dx.UNSET_LONG) == ""
    assert ib_dx.decimalMaxString(ib_dx.UNSET_DECIMAL) == ""
    # The reference's own expressions, not a plain `str()`: a float is written
    # through the eight-place format, so a binary float's exact tail is gone,
    # and a Decimal through the plain format, so the exponent is gone.
    assert ib_dx.floatMaxString(0.1 + 0.2) == "0.3"
    assert ib_dx.floatMaxString(100.0) == "100"
    assert ib_dx.decimalMaxString(Decimal("1E+2")) == "100"
    # A float goes through its string first: taken directly, 0.1 would carry
    # the binary float's exact tail.
    assert ib_dx.decimalMaxString(0.1) == "0.1"


def test_the_account_figures_are_named():
    tags = ib_dx.AccountSummaryTags
    assert tags.NetLiquidation == "NetLiquidation"
    assert tags.SMA == "SMA"
    named = tags.AllTags.split(",")
    assert "NetLiquidation" in named and "DayTradesRemaining" in named
    assert len(named) == len(set(named)), "each figure named once"


def test_the_numbered_kinds_a_program_names():
    # A program asks for a feed and an advisor document by name, and compares
    # a condition's kind against these.
    assert ib_dx.MarketDataTypeEnum.REALTIME == 1
    assert ib_dx.MarketDataTypeEnum.DELAYED == 3
    assert ib_dx.MarketDataTypeEnum.DELAYED_FROZEN == 4
    assert ib_dx.FaDataTypeEnum.GROUPS == 1
    assert ib_dx.FaDataTypeEnum.ALIASES == 3
    assert ib_dx.OrderCondition.Price == 1
    assert ib_dx.OrderCondition.PercentChange == 7
    assert ib_dx.getEnumTypeName(ib_dx.MarketDataTypeEnum, 3) == "DELAYED"
    # A kind whose members carry (the code, the name) answers for the member
    # a caller holds — the reference's own Execution carries members and its
    # helper compares them (ibapi utils.py:244) — trailing space and all.
    # Anything no member is, a bare code included, falls back to the first
    # member's string, exactly as the reference's helper reads it.
    assert ib_dx.getEnumTypeName(ib_dx.OptionExerciseType, ib_dx.OptionExerciseType.Assigned) == "Assigned "
    assert ib_dx.getEnumTypeName(ib_dx.OptionExerciseType, ib_dx.OptionExerciseType.NoneItem) == "None"
    assert ib_dx.getEnumTypeName(ib_dx.OptionExerciseType, 100) == "None"


def test_the_base_a_program_writes_its_own_objects_on():
    # Evaluated as a base class, so it has to exist before that class body runs.
    class Activity(ib_dx.Object):
        pass

    assert str(Activity()) == "Activity"


def test_a_scan_row_and_a_mid_offset_that_means_the_midpoint():
    row = ib_dx.ScanData(rank=1, distance="", benchmark="", projection="", legsStr="")
    assert row.rank == 1 and row.contract is None
    # The row's market name: the reference record carries it as its seventh
    # field and its rendering prints it (ibapi scanner.py:31,44), so a program
    # reading `row.marketName` or building a row under that keyword works.
    assert row.marketName == ""
    contract = ib_dx.Contract()
    contract.symbol, contract.secType, contract.currency = "IBM", "STK", "USD"
    named = ib_dx.ScanData(contract=contract, rank=2, marketName="ISLAND")
    assert named.marketName == "ISLAND"
    assert str(named) == ("Rank: 2, Symbol: IBM, SecType: STK, Currency: USD, "
                          "Distance: , Benchmark: , Projection: , Legs String: , "
                          "MarketName: ISLAND")
    # Not a distance: the offset that says "up to the midpoint" is unbounded.
    assert ib_dx.COMPETE_AGAINST_BEST_OFFSET_UP_TO_MID == float("inf")


def test_a_clock_reading_written_for_a_person():
    assert ib_dx.getTimeStrFromMillis(0) == ""
    assert ib_dx.getTimeStrFromMillis(1772202600000).endswith(".000")


def test_the_reference_client_surface_is_still_exported():
    for name in ("EClient", "EWrapper", "Contract", "Order", "ContractDetails"):
        assert hasattr(ib_dx, name)
