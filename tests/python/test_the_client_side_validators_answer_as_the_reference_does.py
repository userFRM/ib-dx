"""The validators a program written against the reference client calls.

They are pure functions of their arguments there: the order-parameter check
reads the order against the level the session answers, the attached-order
check reads it the same way, and the symbol check reads the string a
connection names. Ported, so a moved program's pre-flight checks answer here
what they answered there — the parameter the session cannot carry, by the
name that client gives it, or nothing.
"""

import ibkr_dx


class Errors(ibkr_dx.EWrapper):
    def __init__(self):
        super().__init__()
        self.seen = []

    def error(self, req_id, error_time, code, msg, advanced_order_reject_json=""):
        self.seen.append((code, msg))


def test_an_order_parameter_the_level_cannot_carry_is_named():
    c = ibkr_dx.EClient(ibkr_dx.EWrapper())
    # Before a session the level reads as nought, as it does there, and every
    # gate asks.
    for stated, named in [
        ("deactivate", "deactivate"),
        ("postOnly", "postOnly"),
        ("allowPreOpen", "allowPreOpen"),
        ("ignoreOpenAuction", "ignoreOpenAuction"),
        ("whatIfType", "whatIfType"),
        ("hedgeMaxSize", "hedgeMaxSize"),
        ("conditionsIncludeOvernight", "conditionsIncludeOvernight"),
    ]:
        order = ibkr_dx.Order()
        setattr(order, stated, 1 if stated in ("whatIfType", "hedgeMaxSize") else True)
        assert c.validateOrderParameters(order) == named, stated
    order = ibkr_dx.Order()
    order.routeMarketableToBbo = False
    assert c.validateOrderParameters(order) == "routeMarketableToBbo"
    assert c.validateOrderParameters(ibkr_dx.Order()) is None

    # At the level a session answers — 217 — the two additional-parameter
    # blocks are carried, and the gates above it still ask.
    c._test_connect("T")
    order = ibkr_dx.Order()
    order.deactivate = True
    order.whatIfType = 1
    assert c.validateOrderParameters(order) is None
    order.whatIfType = 2147483647  # back to the unset value it is born with
    order.hedgeMaxSize = 100
    assert c.validateOrderParameters(order) == "hedgeMaxSize"
    order.hedgeMaxSize = 2147483647
    order.conditionsIncludeOvernight = True
    assert c.validateOrderParameters(order) == "conditionsIncludeOvernight"


def test_an_attached_order_field_below_its_level_is_named():
    c = ibkr_dx.EClient(ibkr_dx.EWrapper())
    assert c.validateAttachedOrdersParameters(ibkr_dx.Order()) is None
    order = ibkr_dx.Order()
    order.slOrderId = 7
    assert c.validateAttachedOrdersParameters(order) == "slOrderId"
    # The keyword the reference client's own signature names reaches the
    # check, and so does its snake_case spelling.
    assert c.validateAttachedOrdersParameters(attachedOrders=order) == "slOrderId"
    assert c.validateAttachedOrdersParameters(attached_orders=order) == "slOrderId"
    order.slOrderId = 2147483647  # back to the unset value it is born with
    order.ptOrderType = "STP"
    assert c.validateAttachedOrdersParameters(order) == "ptOrderType"
    c._test_connect("T")
    # 217 is below the 218 attached orders arrived at: the gate still asks.
    assert c.validateAttachedOrdersParameters(order) == "ptOrderType"


def test_a_host_the_wire_cannot_carry_is_said_under_579():
    w = Errors()
    c = ibkr_dx.EClient(w)
    c._test_connect("T")
    assert c.validateInvalidSymbols("host.name") is None
    assert c.validateInvalidSymbols("host\tname") is None
    assert not w.seen, w.seen
    c.validateInvalidSymbols("hé")
    c.poll()
    assert w.seen == [(579, "Invalid symbol in string - hé")], w.seen
