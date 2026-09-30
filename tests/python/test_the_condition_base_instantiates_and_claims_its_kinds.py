"""The condition base is one a program can build and branch on.

The reference client's OrderCondition takes the kind number, stores it with
the conjunction flag born true, and every condition class derives from it —
so a program instantiates the base directly and isinstance-checks a
condition against it. Here the base was a constants-only class: building it
raised TypeError and every isinstance answered False, so a program branching
on the base type misrouted.
"""

import pytest

from ibkr_dx import (
    ExecutionCondition, MarginCondition, OrderCondition,
    PercentChangeCondition, PriceCondition, TimeCondition, VolumeCondition,
)


def test_the_base_builds_and_joins_as_the_reference_writes_it():
    oc = OrderCondition(OrderCondition.Price)
    assert (oc.condType, oc.isConjunctionConnection, oc.type()) == (1, True, 1)
    assert str(oc) == "<AND>"
    assert repr(oc) == f"{id(oc)}: <AND>"
    assert oc.Or() is oc
    assert (oc.isConjunctionConnection, str(oc)) == (False, "<OR>")
    assert oc.And() is oc
    assert (oc.isConjunctionConnection, str(oc)) == (True, "<AND>")
    # The kind number is a required argument there, so it is one here: a
    # base built with no kind raises in a program written against either.
    with pytest.raises(TypeError):
        OrderCondition()


def test_every_condition_is_an_order_condition():
    for cls in (PriceCondition, TimeCondition, MarginCondition,
                ExecutionCondition, VolumeCondition, PercentChangeCondition):
        assert isinstance(cls(), OrderCondition), cls.__name__
        assert issubclass(cls, OrderCondition), cls.__name__
