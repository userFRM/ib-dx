"""The names a program written against the reference client imports.

That client publishes a handful of plain objects a caller fills in and hands
back — an execution filter, a scanner subscription, a withdrawal — along with
the constants its unset fields carry and the aliases its type annotations
name. A program imports them on its first line, so absent, none of it ran.

Everything here is plain Python, as it is there, because this client reads
these objects by attribute: the shape is the contract, and any object carrying
the same attribute names is already accepted. What each field means is the
venue's business and is documented by the venue; what is written here is the
shape and nothing else. A combination leg, a leg price and a delta-neutral
hedge are not here: a callback hands those back as well as taking them, so
those classes live beside the classes that carry them.
"""

import enum
import math
import sys
from decimal import Decimal

#: An integer field nobody set.
UNSET_INTEGER = 2**31 - 1
#: A floating-point field nobody set.
UNSET_DOUBLE = float(sys.float_info.max)
#: A long field nobody set.
UNSET_LONG = 2**63 - 1
#: A decimal field nobody set.
UNSET_DECIMAL = Decimal(2**127 - 1)
#: A price with no bound above it.
DOUBLE_INFINITY = math.inf
#: How that price is written on the wire.
INFINITY_STR = "Infinity"
#: The request id an unsolicited answer carries.
NO_VALID_ID = -1
#: The largest message the protocol carries, one byte under sixteen megabytes.
MAX_MSG_LEN = 0xFFFFFF

# Aliases the reference client's annotations name. A program evaluates them at
# class-definition time — its callbacks are annotated with them — so a missing
# one is a NameError before the class exists, not a typing nicety.
TickerId = int
OrderId = int
TickType = int
TagValueList = list
SetOfString = set
SetOfFloat = set
SmartComponentMap = dict
ListOfContractDescription = list
ListOfDepthExchanges = list
ListOfNewsProviders = list
ListOfPriceIncrements = list
ListOfFamilyCode = list
ListOfHistoricalTick = list
ListOfHistoricalTickBidAsk = list
ListOfHistoricalTickLast = list
ListOfHistoricalSessions = list
ListOfOrder = list
HistogramDataList = list


def iswrapper(fn):
    """Mark an override as answering a callback.

    A marker and nothing else, as it is on the reference client: its samples
    put it on every override, so a program will not import without it, and it
    changes nothing about the function it is given.
    """
    return fn


class ExecutionFilter:
    """Which of the day's executions a request is asking for.

    Left as it is, it asks for all of them.
    """

    def __init__(self):
        self.clientId = 0
        self.acctCode = ""
        self.time = ""
        self.symbol = ""
        self.secType = ""
        self.exchange = ""
        self.side = ""
        self.lastNDays = UNSET_INTEGER
        self.specificDates = None


class ScannerSubscription:
    """What a scan is looking for.

    `instrument`, `locationCode` and `scanCode` name the scan; the rest bound
    it. A numeric bound left at its unset value is not sent, because a bound of
    the largest number there is would empty the scan rather than widen it.
    """

    def __init__(self):
        self.numberOfRows = -1
        self.instrument = ""
        self.locationCode = ""
        self.scanCode = ""
        self.abovePrice = UNSET_DOUBLE
        self.belowPrice = UNSET_DOUBLE
        self.aboveVolume = UNSET_INTEGER
        self.marketCapAbove = UNSET_DOUBLE
        self.marketCapBelow = UNSET_DOUBLE
        self.moodyRatingAbove = ""
        self.moodyRatingBelow = ""
        self.spRatingAbove = ""
        self.spRatingBelow = ""
        self.maturityDateAbove = ""
        self.maturityDateBelow = ""
        self.couponRateAbove = UNSET_DOUBLE
        self.couponRateBelow = UNSET_DOUBLE
        self.excludeConvertible = False
        self.averageOptionVolumeAbove = UNSET_INTEGER
        self.scannerSettingPairs = ""
        self.stockTypeFilter = ""


class OrderCancel:
    """What a withdrawal states beyond the order it withdraws."""

    def __init__(self):
        self.manualOrderCancelTime = ""
        self.extOperator = ""
        self.manualOrderIndicator = UNSET_INTEGER


class WshEventData:
    """Which corporate events a request is asking for."""

    def __init__(self):
        self.conId = UNSET_INTEGER
        self.filter = ""
        self.fillWatchlist = False
        self.fillPortfolio = False
        self.fillCompetitors = False
        self.startDate = ""
        self.endDate = ""
        self.totalLimit = UNSET_INTEGER


class AccountSummaryTags:
    """The account figures a summary can be asked for, by name."""

    AccountType = "AccountType"
    NetLiquidation = "NetLiquidation"
    TotalCashValue = "TotalCashValue"
    SettledCash = "SettledCash"
    AccruedCash = "AccruedCash"
    BuyingPower = "BuyingPower"
    EquityWithLoanValue = "EquityWithLoanValue"
    PreviousDayEquityWithLoanValue = "PreviousDayEquityWithLoanValue"
    GrossPositionValue = "GrossPositionValue"
    ReqTEquity = "ReqTEquity"
    ReqTMargin = "ReqTMargin"
    SMA = "SMA"
    InitMarginReq = "InitMarginReq"
    MaintMarginReq = "MaintMarginReq"
    AvailableFunds = "AvailableFunds"
    ExcessLiquidity = "ExcessLiquidity"
    Cushion = "Cushion"
    FullInitMarginReq = "FullInitMarginReq"
    FullMaintMarginReq = "FullMaintMarginReq"
    FullAvailableFunds = "FullAvailableFunds"
    FullExcessLiquidity = "FullExcessLiquidity"
    LookAheadNextChange = "LookAheadNextChange"
    LookAheadInitMarginReq = "LookAheadInitMarginReq"
    LookAheadMaintMarginReq = "LookAheadMaintMarginReq"
    LookAheadAvailableFunds = "LookAheadAvailableFunds"
    LookAheadExcessLiquidity = "LookAheadExcessLiquidity"
    HighestSeverity = "HighestSeverity"
    DayTradesRemaining = "DayTradesRemaining"
    Leverage = "Leverage"

    AllTags = ",".join(
        value for name, value in sorted(vars().items())
        if not name.startswith("_") and isinstance(value, str)
    )


def floatMaxString(val: float):
    """A float written for a person, and nothing where nobody set one.

    The reference's own expression (ibapi/utils.py:185): the eight-place
    format, so a binary float's exact tail is gone.
    """
    if val is None:
        return ""
    return (
        f"{val:.8f}".rstrip("0").rstrip(".").rstrip(",") if val != UNSET_DOUBLE else ""
    )


def intMaxString(value):
    """An integer written for a person, and nothing where nobody set one."""
    if value is None or value == UNSET_INTEGER:
        return ""
    return str(value)


def longMaxString(value):
    """A long written for a person, and nothing where nobody set one."""
    if value is None or value == UNSET_LONG:
        return ""
    return str(value)


def decimalMaxString(val: Decimal):
    """A quantity written for a person, and nothing where nobody set one.

    The reference's own expression (ibapi/utils.py:205): a float is taken
    through its string, and the plain format, so no exponent survives.
    """
    val = Decimal(str(val)) if type(val) is float else Decimal(val)
    return f"{val:f}" if val != UNSET_DECIMAL else ""

class Object:
    """The base the reference client's plain objects are written on.

    A program subclasses it for objects of its own — its own sample does — so
    it has to exist before that class body runs. It adds nothing.
    """

    def __str__(self):
        return type(self).__name__

    def __repr__(self):
        return f"{id(self)}: {self}"


class ScanData:
    """One row of a scan, as a callback hands it over."""

    def __init__(self, contract=None, rank=0, distance="", benchmark="",
                 projection="", legsStr=""):
        self.contract = contract
        self.rank = rank
        self.distance = distance
        self.benchmark = benchmark
        self.projection = projection
        self.legsStr = legsStr


class OrderCondition:
    """What kind of thing an order's condition watches.

    The numbers the venue gives each kind, which a program compares against
    `condType` and passes when it builds one.
    """

    Price = 1
    Time = 3
    Margin = 4
    Execution = 5
    Volume = 6
    PercentChange = 7


class MarketDataTypeEnum:
    """Which feed a subscription asks for, by the venue's numbering."""

    REALTIME = 1
    FROZEN = 2
    DELAYED = 3
    DELAYED_FROZEN = 4


class FaDataTypeEnum:
    """Which advisor document a request is asking for."""

    GROUPS = 1
    ALIASES = 3


class Enum:
    """The small numbered-name holder the reference client builds its plain
    enums on (ibapi enum_implem.py): each name given becomes an attribute
    holding its position, and a position reads back as its name.

    A program star-importing `common` uses it unqualified there — the reference
    client's own module imports it — so it has to exist before that program's
    class bodies run.
    """

    def __init__(self, *args):
        self.idx2name = {}
        for idx, name in enumerate(args):
            setattr(self, name, idx)
            self.idx2name[idx] = name

    def toStr(self, idx):
        return self.idx2name.get(idx, "NOTFOUND")


#: Which side of a trade added the liquidity, by the venue's numbering. The
#: last name is spelled as the reference client spells it.
LiquiditiesEnum = Enum("None", "Added", "Remove", "RoudedOut")

#: The aliases the reference client's annotations name for the three kinds.
FaDataType = int
MarketDataType = int
Liquidities = int


#: A mid-offset that means "up to the midpoint" rather than a distance.
COMPETE_AGAINST_BEST_OFFSET_UP_TO_MID = DOUBLE_INFINITY

# The module-level names the reference client's `order`, `contract`, `scanner`
# and `news` modules publish, which a program star-importing one of them uses
# unqualified: its own `Order.__init__` writes `self.origin = CUSTOMER`.
(CUSTOMER, FIRM, UNKNOWN) = range(3)
(AUCTION_UNSET, AUCTION_MATCH, AUCTION_IMPROVEMENT, AUCTION_TRANSPARENT) = range(4)
(SAME_POS, OPEN_POS, CLOSE_POS, UNKNOWN_POS) = range(4)
NO_ROW_NUMBER_SPECIFIED = -1
NEWS_MSG = 1
EXCHANGE_AVAIL_MSG = 2
EXCHANGE_UNAVAIL_MSG = 3


def getTimeStrFromMillis(time):
    """A millisecond clock reading written for a person, and nothing for none."""
    if not time or time <= 0:
        return ""
    import datetime

    stamp = datetime.datetime.fromtimestamp(time / 1000.0)
    return stamp.strftime("%b %d, %Y %H:%M:%S.%f")[:-3]


def getEnumTypeName(cls, value):
    """The name a numbered kind goes by, or the first one where it names none.

    A kind whose members carry (the code, the name) — the exercise and fund
    kinds — answers the string the member itself carries, "Assigned " with its
    trailing space and all; a plain numbered class answers the attribute's name.
    The reference client's own helper (ibapi utils.py:244) reads that way, and
    `Execution.__str__` (execution.py:80) spells an exercise event with it, so
    a log line or CSV column a moved program builds on the helper reads the
    same here.
    """
    first = None
    for name, held in vars(cls).items():
        if name.startswith("_"):
            continue
        carried = getattr(held, "value", None)
        if isinstance(carried, tuple):
            # ponytail: first non-underscore attribute wins the fallback; for
            # the member-carrying classes that is the first member, as there.
            first = carried[1] if first is None else first
            # The member itself is compared, as the reference compares it —
            # its Execution carries members, so a code no member is names
            # nothing and takes the fallback.
            if held == value:
                return carried[1]
        else:
            first = name if first is None else first
            if held == value:
                return name
    return first if first is not None else ""


class OptionExerciseType(enum.Enum):
    """How an option position came to be exercised, as the reference client
    numbers it: each member is (the code, the name)."""
    NoneItem = (-1, "None")
    Exercise = (1, "Exercise")
    Lapse = (2, "Lapse")
    DoNothing = (3, "DoNothing")
    Assigned = (100, "Assigned ")
    AutoexerciseClearing = (101, "AutoexerciseClearing")
    Expired = (102, "Expired")
    Netting = (103, "Netting")
    AutoexerciseTrading = (200, "AutoexerciseTrading")


class FundAssetType(enum.Enum):
    """A fund's asset class, as the reference client lists it: (the code, the name)."""
    NoneItem = ("None", "None")
    Others = ("000", "Others")
    MoneyMarket = ("001", "Money Market")
    FixedIncome = ("002", "Fixed Income")
    MultiAsset = ("003", "Multi-asset")
    Equity = ("004", "Equity")
    Sector = ("005", "Sector")
    Guaranteed = ("006", "Guaranteed")
    Alternative = ("007", "Alternative")


class FundDistributionPolicyIndicator(enum.Enum):
    """Whether a fund accumulates or pays out, as the reference client lists it."""
    NoneItem = ("None", "None")
    AccumulationFund = ("N", "Accumulation Fund")
    IncomeFund = ("Y", "Income Fund")


def member_for(cls, code):
    """The member whose code this is, or the first listed where none is: the
    reference client's own lookup, which is how it reads a code off the wire."""
    for member in cls:
        if member.value[0] == code:
            return member
    return next(iter(cls))


def isValidFloatValue(val: float) -> bool:
    """Whether a float field is one somebody set (ibapi utils.py:173)."""
    return val != UNSET_DOUBLE


def isValidIntValue(val: int) -> bool:
    """Whether an integer field is one somebody set."""
    return val != UNSET_INTEGER


def isValidLongValue(val: int) -> bool:
    """Whether a long field is one somebody set."""
    return val != UNSET_LONG


def isValidDecimalValue(val: Decimal) -> bool:
    """Whether a decimal field is one somebody set."""
    return val != UNSET_DECIMAL


def isAsciiPrintable(val):
    """Whether every character of a string can go to a gateway.

    The reference client's own expression (ibapi utils.py:201): the printable
    characters, plus the tab, the line feed and the carriage return.
    """
    return all(ord(c) >= 32 and ord(c) < 127 or ord(c) == 9 or ord(c) == 10 or ord(c) == 13 for c in val)


def isPegBenchOrder(orderType: str):
    """Whether an order type is a peg-to-benchmark one."""
    return orderType in ("PEG BENCH", "PEGBENCH")


def isPegMidOrder(orderType: str):
    """Whether an order type is a peg-to-midpoint one."""
    return orderType in ("PEG MID", "PEGMID")


def isPegBestOrder(orderType: str):
    """Whether an order type is a peg-to-best one."""
    return orderType in ("PEG BEST", "PEGBEST")


def currentTimeMillis():
    """The wall clock in milliseconds, rounded as the reference client rounds it."""
    import time

    return round(time.time() * 1000)


def listOfValues(cls):
    """The members of an enum class, in declaration order."""
    return list(map(lambda c: c, cls))


#: The member whose first element a string is, or the first listed where none
#: is — the lookup `member_for` already is, under the name the reference
#: client's decoder reads a fund's kind through (ibapi utils.py:238).
getEnumTypeFromString = member_for

class RealTimeBar:
    """One five-second bar, as a callback hands it over.

    A program builds one of these itself in its own `realtimeBar` override, so
    it has to be constructible the way that client's is.
    """

    def __init__(self, time=0, endTime=-1, open_=0.0, high=0.0, low=0.0,
                 close=0.0, volume=UNSET_DECIMAL, wap=UNSET_DECIMAL, count=0):
        self.time = time
        self.endTime = endTime
        self.open_ = open_
        self.high = high
        self.low = low
        self.close = close
        self.volume = volume
        self.wap = wap
        self.count = count


class HistogramData:
    """How much traded at one price, over the window asked about."""

    def __init__(self):
        self.price = 0.0
        self.size = UNSET_DECIMAL


class FamilyCode:
    """An account and the family it belongs to."""

    def __init__(self):
        self.accountID = ""
        self.familyCodeStr = ""


class HistoricalSession:
    """One session in the schedule a contract trades on."""

    def __init__(self):
        self.startDateTime = ""
        self.endDateTime = ""
        self.refDate = ""


class CodeMsgPair:
    """An error number and the standing words that go with it.

    The reference client's `errors` module publishes one of these per number
    it reports itself, and a program compares a callback's number against
    `errors.NOT_CONNECTED.errorCode` rather than a bare 504.
    """

    def __init__(self, code, msg):
        self.errorCode = code
        self.errorMsg = msg


#: A second connection asked for while one is up.
ALREADY_CONNECTED = CodeMsgPair(501, "Already connected.")
#: A request made with no session to send it on.
NOT_CONNECTED = CodeMsgPair(504, "Not connected")
#: A verification message this client answers itself.
BAD_MESSAGE = CodeMsgPair(508, "Bad message")


class OrderStatus(enum.Enum):
    """The states an order goes through, as the callbacks state them.

    A program compares the status an `orderStatus` or `openOrder` callback
    carried against these, so the strings are the contract. `get` reads the
    string a callback carried, whichever way round its letters are, and
    answers `Unknown` for one no member names.
    """

    ApiPending = "ApiPending"
    ApiCancelled = "ApiCancelled"
    PreSubmitted = "PreSubmitted"
    PendingCancel = "PendingCancel"
    Cancelled = "Cancelled"
    Submitted = "Submitted"
    Filled = "Filled"
    Inactive = "Inactive"
    PendingSubmit = "PendingSubmit"
    Unknown = "Unknown"

    def __str__(self) -> str:
        return self.value

    @classmethod
    def get(cls, api_string: str) -> "OrderStatus":
        """The member a status string names, case aside, or `Unknown`."""
        if not api_string:
            return cls.Unknown
        for member in cls:
            if member.value.lower() == api_string.strip().lower():
                return member
        return cls.Unknown

    def is_active(self) -> bool:
        """Whether further fills, or a withdrawal, are still possible."""
        return self in (
            OrderStatus.PreSubmitted,
            OrderStatus.PendingCancel,
            OrderStatus.Submitted,
            OrderStatus.PendingSubmit,
        )

    def is_terminal(self) -> bool:
        """Whether the order will not move again."""
        return self in (
            OrderStatus.Filled,
            OrderStatus.Cancelled,
            OrderStatus.Inactive,
            OrderStatus.ApiCancelled,
        )

