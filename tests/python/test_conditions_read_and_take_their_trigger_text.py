"""A condition reads and takes its trigger figure as the reference's text.

The reference client gives every operator condition a valueToString /
setValueFromString pair — the documented way to read a trigger string back
into a condition — and neither existed here, so a program rendering or
rebuilding condition text crashed with AttributeError on every kind. The
execution condition carries no pair there (it derives from the base, not
from the operator condition), so it carries none here.

Run: pytest tests/python/test_conditions_read_and_take_their_trigger_text.py -v
"""

import pytest

from ib_dx import (
    ExecutionCondition, MarginCondition, PercentChangeCondition,
    PriceCondition, TimeCondition, VolumeCondition,
)


def test_each_condition_reads_its_trigger_as_the_reference_text():
    assert PriceCondition(0, 756733, "SMART", True, 200.0).valueToString() == "200.0"
    assert PriceCondition().valueToString() == "None"
    assert MarginCondition(True, 25).valueToString() == "25"
    assert PercentChangeCondition(756733, "SMART", False, 5.0).valueToString() == "5.0"
    # A percent nobody stated reads at the unset double, as it is born there.
    assert PercentChangeCondition().valueToString() == "1.7976931348623157e+308"
    assert VolumeCondition(756733, "SMART", True, 1_000_000).valueToString() == "1000000"
    assert TimeCondition(False, "20260101 09:30:00").valueToString() == "20260101 09:30:00"
    # The time comes back as the object itself — nothing unstated is "None".
    assert TimeCondition().valueToString() is None


def test_each_condition_takes_the_reference_text_back():
    p = PriceCondition()
    p.setValueFromString("2.5")
    assert p.price == 2.5

    m = MarginCondition()
    m.setValueFromString("30")
    assert m.percent == 30
    assert m.valueToString() == "30"
    # The percent is an integer here — the engine holds it as one — so a
    # fraction is refused rather than truncated or silently re-rendered.
    with pytest.raises(ValueError):
        m.setValueFromString("30.5")

    v = VolumeCondition()
    v.setValueFromString("1000")
    assert v.volume == 1000
    assert v.valueToString() == "1000"

    pc = PercentChangeCondition()
    pc.setValueFromString("5.5")
    assert pc.changePercent == 5.5

    t = TimeCondition()
    t.setValueFromString("20260101 09:30:00")
    assert t.time == "20260101 09:30:00"
    assert t.valueToString() == "20260101 09:30:00"


def test_the_execution_condition_carries_no_text_pair_as_there():
    e = ExecutionCondition("STK", "SMART", "AAPL")
    with pytest.raises(AttributeError):
        e.valueToString()
    with pytest.raises(AttributeError):
        e.setValueFromString("x")
