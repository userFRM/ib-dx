"""A request states what the caller stated, not a value filled in for them.

The reference client requires the underlying's type on an option-chain request
and holds no exchange on a condition nobody named one on. Defaulted here, a
caller who left either off was answered about something else and told nothing.

The book request's own contract is checked beside the engine command it makes,
in the Rust tests.
"""

import pytest

import ib_dx


def test_option_chains_need_the_underlying_type():
    c = ib_dx.EClient(ib_dx.EWrapper())
    with pytest.raises(TypeError):
        c.req_sec_def_opt_params(3, "SPY")


def test_a_condition_states_no_field_the_caller_did_not():
    for cond in (ib_dx.PriceCondition(con_id=1, price=1.0),
                 ib_dx.VolumeCondition(con_id=1, volume=1),
                 ib_dx.PercentChangeCondition(con_id=1, change_percent=1.0)):
        assert cond.exchange is None, f"{type(cond).__name__} invented {cond.exchange!r}"
        assert cond.is_more is None, f"{type(cond).__name__} invented {cond.is_more!r}"
    # Every field the reference client births unset is unset here: a value
    # invented as a default goes on the wire as one the caller stated.
    assert ib_dx.PriceCondition().con_id is None
    assert ib_dx.PriceCondition().trigger_method is None
    assert ib_dx.TimeCondition().time is None
    assert ib_dx.MarginCondition().percent is None
    assert ib_dx.VolumeCondition().volume is None
    assert ib_dx.ExecutionCondition().sec_type is None
    assert ib_dx.ExecutionCondition().symbol is None
    # The percent change is born at the reference's own unset marker for a
    # double, as it is born there, rather than at `None`.
    assert ib_dx.PercentChangeCondition().change_percent == ib_dx.UNSET_DOUBLE
    # What every class is born holding, the reference births it holding too.
    assert ib_dx.PriceCondition().is_conjunction_connection is True
