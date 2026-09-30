"""The condition surface the reference client writes, laid over the engine's.

The six condition classes are the engine's own; the reference client's
programs build them through their base, `OrderCondition` (ibapi
order_condition.py). The base is plain Python and the six are not its
subclasses, so each is registered against it — `isinstance` and `issubclass`
then answer as the reference's own inheritance answers.
"""

#: The six kinds, in the order the reference client's module defines them.
_KINDS = (
    "PriceCondition",
    "TimeCondition",
    "MarginCondition",
    "ExecutionCondition",
    "VolumeCondition",
    "PercentChangeCondition",
)


def install(surface) -> None:
    """Claim the six condition classes for the base they answer to."""
    base = surface["OrderCondition"]
    for name in _KINDS:
        base.register(surface[name])
