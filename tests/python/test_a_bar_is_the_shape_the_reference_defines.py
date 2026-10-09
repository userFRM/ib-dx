"""A bar handed to a program is the shape the reference client defines.

Its class states eight fields — date, open, high, low, close, volume, wap
and barCount — and nothing else. This client once carried two of its own on
the same surface: the timezone the reply stated and the end of an aggregated
bar, engine figures presented to a program as venue data, so a walk of the
reference's field set diverged. Both stay engine-side, where the client
already tracks the zone per request for its own time formatting.
"""

import pytest

import ib_dx


def test_a_bar_carries_exactly_the_reference_fields():
    bar = ib_dx.BarData("20260309", 1.0, 2.0, 0.5, 1.5, 10, 1.2, 3)
    assert not hasattr(bar, "timezone")
    assert not hasattr(bar, "end")
    # And its constructor takes no more than the reference's eight values.
    with pytest.raises(TypeError):
        ib_dx.BarData("20260309", 1.0, 2.0, 0.5, 1.5, 10, 1.2, 3, "US/Eastern")
