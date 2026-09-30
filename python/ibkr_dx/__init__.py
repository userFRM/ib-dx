"""An Interactive Brokers client with no gateway and no JVM.

``EClient``/``EWrapper`` carry the TWS API's shape — a request under an id, an
answer later on a callback. A program written against ``ibapi`` moves here by
changing its imports and its connect call; where an answer differs from a
gateway's, the Limits page of the documentation says so.
"""

import inspect
from importlib.metadata import version as _version

from .ibkr_dx import *  # noqa: F401,F403
from .ibkr_dx import __doc__ as _ext_doc  # noqa: F401

from ._settings import UNAVAILABLE, configure, describe, settings  # noqa: F401

#: The installed release, as the reference client's package states its own.
__version__ = _version("ibkr-dx")

# The plain objects and constants a program written against the reference
# client imports on its first line. Read by attribute here, so the shape is the
# whole of what they are.
from ._reference_shapes import (  # noqa: F401
    ALREADY_CONNECTED,
    BAD_MESSAGE,
    DOUBLE_INFINITY,
    INFINITY_STR,
    MAX_MSG_LEN,
    NOT_CONNECTED,
    NO_VALID_ID,
    UNSET_DECIMAL,
    UNSET_DOUBLE,
    UNSET_INTEGER,
    UNSET_LONG,
    COMPETE_AGAINST_BEST_OFFSET_UP_TO_MID,
    CUSTOMER,
    FIRM,
    UNKNOWN,
    AUCTION_UNSET,
    AUCTION_MATCH,
    AUCTION_IMPROVEMENT,
    AUCTION_TRANSPARENT,
    SAME_POS,
    OPEN_POS,
    CLOSE_POS,
    UNKNOWN_POS,
    NO_ROW_NUMBER_SPECIFIED,
    NEWS_MSG,
    EXCHANGE_AVAIL_MSG,
    EXCHANGE_UNAVAIL_MSG,
    AccountSummaryTags,
    BadMessage,
    CodeMsgPair,
    Enum,
    ExecutionFilter,
    FaDataType,
    FaDataTypeEnum,
    FundAssetType,
    FundDistributionPolicyIndicator,
    FamilyCode,
    HistogramData,
    HistogramDataList,
    HistoricalSession,
    ListOfContractDescription,
    ListOfDepthExchanges,
    ListOfFamilyCode,
    ListOfHistoricalSessions,
    ListOfHistoricalTick,
    ListOfHistoricalTickBidAsk,
    ListOfHistoricalTickLast,
    ListOfNewsProviders,
    ListOfOrder,
    ListOfPriceIncrements,
    Liquidities,
    LiquiditiesEnum,
    MarketDataType,
    MarketDataTypeEnum,
    Object,
    OptionExerciseType,
    OrderCancel,
    OrderCondition,
    OrderId,
    OrderStatus,
    RealTimeBar,
    ScanData,
    ScannerSubscription,
    SetOfFloat,
    SetOfString,
    SmartComponentMap,
    TagValueList,
    TickerId,
    TickType,
    WshEventData,
    currentTimeMillis,
    decimalMaxString,
    decode,
    floatMaxString,
    getEnumTypeFromString,
    getEnumTypeName,
    getTimeStrFromMillis,
    intMaxString,
    isAsciiPrintable,
    isPegBenchOrder,
    isPegBestOrder,
    isPegMidOrder,
    isValidDecimalValue,
    isValidFloatValue,
    isValidIntValue,
    isValidLongValue,
    iswrapper,
    listOfValues,
    longMaxString,
    printProtoSingleLine,
)

#: What a fill cost, under the name the reference client gave it before the
#: fees were reported beside the commission. A program written against those
#: releases imports `commission_report.CommissionReport`, so both spellings are
#: published and name the one class.
CommissionReport = CommissionAndFeesReport  # noqa: F405


def _reference_name(ours: str) -> str:
    """What the reference client calls the call or callback this one calls `ours`.

    A capital after each underscore, except for those it spells with the
    letters run together, with an acronym in capitals, or with another word.
    The engine keeps the callbacks among them, beside the code that delivers a
    callback under that client's name.
    """
    run_together = {
        "real_time_bar": "realtimeBar",
        "receive_fa": "receiveFA",
        "replace_fa_end": "replaceFAEnd",
        "req_pnl": "reqPnL",
        "cancel_pnl": "cancelPnL",
        "req_pnl_single": "reqPnLSingle",
        "cancel_pnl_single": "cancelPnLSingle",
        "req_pnl_proto_buf": "reqPnLProtoBuf",
        "cancel_pnl_proto_buf": "cancelPnLProtoBuf",
        "req_pnl_single_proto_buf": "reqPnLSingleProtoBuf",
        "cancel_pnl_single_proto_buf": "cancelPnLSingleProtoBuf",
        "req_fa_proto_buf": "reqFAProtoBuf",
        "replace_fa_proto_buf": "replaceFAProtoBuf",
        "receive_fa_proto_buf": "receiveFAProtoBuf",
        "replace_fa_end_proto_buf": "replaceFAEndProtoBuf",
        "req_head_time_stamp_proto_buf": "reqHeadTimestampProtoBuf",
        "cancel_head_time_stamp_proto_buf": "cancelHeadTimestampProtoBuf",
        "request_fa": "requestFA",
        "replace_fa": "replaceFA",
        "set_connect_options": "setConnectionOptions",
        "tick_efp": "tickEFP",
        "verify_message_api": "verifyMessageAPI",
        "verify_and_auth_message_api": "verifyAndAuthMessageAPI",
    }
    if ours in run_together:
        return run_together[ours]
    head, *rest = ours.split("_")
    return head + "".join(word[:1].upper() + word[1:] for word in rest)


def _answer_to_both_spellings(cls) -> None:
    """Put the reference client's spelling on the class, not only on instances.

    Resolved on the instance alone, `super().tickPrice(...)` does not find it:
    `super` walks the remaining classes' own contents and never asks an
    instance hook. A program written against that client calls `super()` first
    in nearly every override — its own sample does — so the rest of each
    override never ran, and the dispatch loop logged the failure and carried on
    without it.

    A method both clients spell the same (`error`, `pnl`, `position`) is
    wrapped in place rather than skipped: the reference client's keywords for
    it are its own words — `reqId`, `position` — and only the stand-in remaps
    them, so without the wrap a keyword call under either spelling raised.
    """
    for ours in list(vars(cls)):
        if ours.startswith("_"):
            continue
        theirs = _reference_name(ours)
        if theirs != ours and hasattr(cls, theirs):
            continue
        under_ours = getattr(cls, ours)
        # A value read off the instance, not a call: `clientId` is what
        # `client_id` answers. Wrapped as a call, reading it handed back the
        # wrapper itself.
        if not callable(under_ours):
            if theirs == ours:
                continue
            setattr(cls, theirs, property(
                lambda self, ours=ours: getattr(self, ours), doc=under_ours.__doc__,
            ))
            continue
        setattr(cls, theirs, _standing_in_front_of(under_ours, cls, theirs))


def _one_word(name: str) -> str:
    """A parameter name with nothing in it but its letters, in lower case.

    `useRTH` and `use_rth` are the same parameter, and so are `acctCode` and
    `acct_code`; the underscores and the capitals are the only difference, and
    an acronym makes the capitals unguessable in either direction.
    """
    return name.replace("_", "").lower()


#: Parameters this client names something else entirely, by letters alone. The
#: rest of the reference client's names differ from ours only in underscores and
#: capitals, which `_one_word` settles; these are different words (or, for
#: `orderBound`, the same words paired differently), so nothing but a list of
#: them will do. A name absent from the method it is passed to is handed on
#: untouched, and the method refuses it as before. Where a method has a
#: parameter of its own that spells one of these keys, this list answers the
#: reference client's word for it only where another of its words claims the
#: parameter the letters alone fold that key to, as `orderBound`'s parameters
#: are the reference's in a different order of words.
_THEIR_WORD_FOR_IT = {
    "tickerid": "req_id",
    "fadata": "fa_data_type",
    "implvoloptions": "implied_vol_options",
    "requestid": "req_id",
    "val": "value",
    "position": "pos",
    "time": "date",
    "cxml": "xml",
    "accountname": "account",
    "newsmessage": "message",
    "originexch": "orig_exchange",
    "totaldividends": "implied_future",
    "xyzchallange": "xyz_challenge",
    "permid": "order_id",
    "clientid": "api_client_id",
    "orderid": "api_order_id",
}


def _under_our_names(method):
    """What each of this method's parameters is called, by its letters alone.

    Read off the method itself rather than composed from a rule: a rule that
    turns `use_rth` into `useRth` misses `useRTH`, which is what the reference
    client calls it and therefore what a caller writes.

    Empty where the method states no parameters this can read, in which case a
    keyword is passed on untouched and the method itself refuses it.
    """
    try:
        params = inspect.signature(method).parameters
    except (TypeError, ValueError):
        return {}
    ours = {_one_word(name): name for name in params if name != "self"}
    # A word on the list moves a parameter the letters alone already claim
    # only where the list also claims the one they fold it to: `orderBound`
    # pairs the reference's words differently — its `permId` arrives on
    # `order_id`, so its `orderId` has to move to `api_order_id`. Elsewhere
    # the fold owns the keyword: `order_status` carries a `perm_id` of its
    # own, no word on the list claims it, and moving `permId` to `order_id`
    # collided with the caller's `orderId` — the remap was thrown away whole
    # and the reference's own keywords were refused.
    claimed = {mine for _, mine in _THEIR_WORD_FOR_IT.items() if mine in params}
    for theirs, mine in _THEIR_WORD_FOR_IT.items():
        if mine in params and (theirs not in ours or ours[theirs] in claimed):
            ours[theirs] = mine
    return ours


def _standing_in_front_of(method, cls, theirs: str):
    """The reference spelling, as a function that calls the method.

    Not the method under a second name. Delivery tries the reference spelling
    first, so a base class answering to it stands in front of a subclass that
    overrode only this client's spelling — and the engine has to tell the base's
    answer from the caller's to pass it over. A bound Python function carries
    `__func__` and the method's own bound form carries nothing that names it,
    so the function is what makes the two tellable apart.

    It also takes a keyword under the reference client's spelling of it. A
    program written against that client passes them by name — its own
    documentation gives ten of them for one request — and every one of those
    names was refused, so calling the reference spelling of a method with the
    reference spelling of its arguments raised.
    """
    ours_by_letters = _under_our_names(method)

    def theirs_calls_ours(self, *args, **kwargs):
        if kwargs:
            remapped = {
                ours_by_letters.get(_one_word(given), given): value
                for given, value in kwargs.items()
            }
            # A caller who gave one figure under both spellings is not
            # silently folded into one: the method itself refuses what it
            # was handed, as it always has.
            if len(remapped) == len(kwargs):
                kwargs = remapped
        return method(self, *args, **kwargs)

    theirs_calls_ours.__name__ = theirs
    theirs_calls_ours.__qualname__ = f"{cls.__name__}.{theirs}"
    theirs_calls_ours.__doc__ = method.__doc__
    # What `inspect.signature` reads the method's own through.
    theirs_calls_ours.__wrapped__ = method
    return theirs_calls_ours


# The reference client's second encoding: the *ProtoBuf family on both
# surfaces, routing to the text requests, and `useProtoBuf` answering False.
# Installed before the spellings are paired, so the reference names are
# generated for what it puts there as for any other method.
from . import _protobuf_shapes as _protobuf_shapes_module  # noqa: E402

_protobuf_shapes_module.install(globals())
del _protobuf_shapes_module

for _surface in (EWrapper, EClient):  # noqa: F405
    _answer_to_both_spellings(_surface)
del _surface

# The price condition's trigger methods, which the reference client holds on
# the class itself (ibapi order_condition.py), built on that client's Enum —
# the official sample spells every method through it, so a condition built
# that way has to find it there.
PriceCondition.TriggerMethodEnum = Enum(  # noqa: F405
    "Default", "DoubleBidAsk", "Last", "DoubleLast", "BidAsk",
    "N/A1", "N/A2", "LastBidAsk", "MidPoint",
)

# The condition base claims the six condition classes, so a program's
# isinstance checks against it answer as the reference client's own
# inheritance answers.
from . import _condition_shapes as _condition_shapes_module  # noqa: E402

_condition_shapes_module.install(globals())
del _condition_shapes_module

# The record classes print the reference client's own sentences, with unset
# values blank rather than as this client's markers.
from . import _record_shapes as _record_shapes_module  # noqa: E402

_record_shapes_module.install(globals())
del _record_shapes_module

# The reference client's own module names, laid over what this package
# publishes, so its import lines resolve under the one rename a person would
# guess at. Bound as attributes as well as registered, because a program writes
# both `from ibkr_dx import wrapper` and `from ibkr_dx.wrapper import EWrapper`.
from . import _layout as _layout_module  # noqa: E402

globals().update(_layout_module.install(dict(globals())))
del _layout_module

# What `from ibkr_dx import *` brings. The extension module is bound on this
# package by the star-import above, so `dir()` names it too: left in, the star
# import rebinds the caller's own `ibkr_dx` to that submodule and `ibkr_dx.configure` stops
# existing. The layout's modules go the same way — `from ibapi import *` brings
# none of them there, and a script that had its own `order` or `contract` would
# lose it to ours. So: no modules, and nothing this file merely imported to do
# its own work.
import types as _types

__all__ = [
    n for n, held in sorted(globals().items())
    if not n.startswith("_")
    and n != "ibkr_dx"
    and not isinstance(held, _types.ModuleType)
]
