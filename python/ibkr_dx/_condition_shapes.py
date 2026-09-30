"""The condition surface the reference client writes, laid over the engine's.

The six condition classes are the engine's own; the reference client's
programs build them through their base, `OrderCondition`, read each
condition's trigger figure as text and take one back (ibapi
order_condition.py). The base is plain Python and the six are not its
subclasses, so each is registered against it — `isinstance` and `issubclass`
then answer as the reference's own inheritance answers — and each carries the
reference's own text algorithms for the field it watches. The execution
condition carries no text pair, as there: it derives from the base, not from
the operator condition. The wire builders stay out — the wire here is Rust.
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


def _price_value_to_string(self) -> str:
    return str(self.price)


def _price_set_value_from_string(self, text: str) -> None:
    self.price = float(text)


def _margin_value_to_string(self) -> str:
    return str(self.percent)


def _margin_set_value_from_string(self, text: str) -> None:
    # The reference stores the float its own coercion makes; the percent here
    # is the engine's integer, so a whole figure lands as one and a fraction
    # is refused rather than truncated.
    value = float(text)
    if value != int(value):
        raise ValueError(f"a margin cushion percent states a fraction: {text}")
    self.percent = int(value)


def _percent_change_value_to_string(self) -> str:
    return str(self.changePercent)


def _percent_change_set_value_from_string(self, text: str) -> None:
    self.changePercent = float(text)


def _volume_value_to_string(self) -> str:
    return str(self.volume)


def _volume_set_value_from_string(self, text: str) -> None:
    self.volume = int(text)


def _time_value_to_string(self) -> str:
    return self.time


def _time_set_value_from_string(self, text: str) -> None:
    self.time = text


def install(surface) -> None:
    """Claim the six condition classes for the base they answer to."""
    base = surface["OrderCondition"]
    for name in _KINDS:
        base.register(surface[name])

    for name, reads, takes in (
        ("PriceCondition", _price_value_to_string, _price_set_value_from_string),
        ("MarginCondition", _margin_value_to_string, _margin_set_value_from_string),
        ("PercentChangeCondition", _percent_change_value_to_string, _percent_change_set_value_from_string),
        ("VolumeCondition", _volume_value_to_string, _volume_set_value_from_string),
        ("TimeCondition", _time_value_to_string, _time_set_value_from_string),
    ):
        cls = surface[name]
        cls.valueToString = reads
        cls.setValueFromString = takes
