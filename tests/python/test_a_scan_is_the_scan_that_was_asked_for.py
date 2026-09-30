"""A scan runs on what the caller described, or it does not run.

The fields naming a scan were read with a fallback, so one that could not be
read became a different scan entirely: the top gaining US stocks, under the
caller's own request id and answered as though it were theirs. A field left
off is still the default, which is what the reference client does with one.
"""

import ib_async
import pytest

import ibkr_dx


class Errors(ibkr_dx.EWrapper):
    def __init__(self):
        super().__init__()
        self.errors = []

    def error(self, reqId, errorTime, errorCode, errorString, advancedOrderRejectJson=""):
        self.errors.append((reqId, errorCode, errorString))


def _client():
    w = Errors()
    c = ibkr_dx.EClient(w)
    c._test_connect("T")
    return w, c


def test_an_ordinary_subscription_is_sent():
    """Their own object states no rows as a negative number, which is not an
    unreadable value: it means the venue picks."""
    w, c = _client()
    c.req_scanner_subscription(1, ib_async.ScannerSubscription())
    assert w.errors == [], w.errors


def test_a_field_stated_and_unreadable_is_refused():
    w, c = _client()
    sub = ib_async.ScannerSubscription()
    sub.scanCode = 42
    c.req_scanner_subscription(2, sub)
    c.poll()
    assert [(rid, code) for rid, code, _ in w.errors] == [(2, 321)], w.errors
    assert "scanCode" in w.errors[0][2]


@pytest.mark.parametrize("field,req_id", [
    ("scanCode", 6),
    ("abovePrice", 7),
    ("excludeConvertible", 8),
    ("stockTypeFilter", 9),
    ("scannerSettingPairs", 10),
])
def test_a_field_stated_none_is_refused_at_the_send(field, req_id):
    """The reference client's encoder raises on None and the request's
    catch-all reports the send error under the caller's request id, with
    nothing sent. Every field answers this way, the filter fields included:
    the subscription defaults all of them to a non-None value, so a field
    carrying None is one the caller stated."""
    w, c = _client()
    sub = ib_async.ScannerSubscription()
    setattr(sub, field, None)
    c.req_scanner_subscription(req_id, sub)
    c.poll()
    assert w.errors == [
        (req_id, 524, "Request Scanner Subscription Sending Error - Cannot send None to TWS"),
    ], w.errors


def test_a_field_left_off_takes_the_default():
    """Anything shaped like a subscription works, as the reference client's
    own duck-typing allows."""
    class Sparse:
        scanCode = "HOT_BY_VOLUME"

    w, c = _client()
    c.req_scanner_subscription(3, Sparse())
    assert w.errors == [], w.errors


def test_a_denied_scan_code_is_refused_under_its_own_code():
    """A login feature flag denying the scan code refuses the subscription at
    the door, under its own code and in a gateway's words, and the venue is
    asked nothing."""
    w, c = _client()
    c._test_set_enabled_features(["DENY_APISCAN_HOT_BY_VOLUME"])
    sub = ib_async.ScannerSubscription()
    sub.scanCode = "HOT_BY_VOLUME"
    c.req_scanner_subscription(4, sub)
    c.poll()
    assert w.errors == [(4, 10359, "Scan code HOT_BY_VOLUME is not allowed")], w.errors


def test_a_denied_filter_is_refused_under_its_own_code():
    """A filter in a flagged group is refused the same way, under its own
    code, naming the filter tag the subscription carries."""
    w, c = _client()
    c._test_set_enabled_features(["DENY_APISCAN_priceAbove"])
    sub = ib_async.ScannerSubscription()
    sub.scanCode = "TOP_PERC_GAIN"
    sub.abovePrice = 10.0
    c.req_scanner_subscription(5, sub)
    c.poll()
    assert w.errors == [(5, 10360, "Scan filter priceAbove is not allowed")], w.errors
