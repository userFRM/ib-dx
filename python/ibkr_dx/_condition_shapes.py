"""The condition surface the reference client writes, laid over the engine's.

The six condition classes are the engine's own; the reference client's
programs build them through their base, `OrderCondition`, read each
condition's trigger figure as text and take one back, and print each as a
sentence chained through the shared operator and contract wordings (ibapi
order_condition.py). The base is plain Python and the six are not its
subclasses, so each is registered against it — `isinstance` and `issubclass`
then answer as the reference's own inheritance answers — and each carries the
reference's own text algorithms for the field it watches, its own sentence
and the Object base's repr. The execution condition carries no text pair, as
there: it derives from the base, not from the operator condition. The wire
builders stay out — the wire here is Rust.
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


# The sentences, chained as the reference chains its classes: the operator
# wording every operator condition shares, the contract wording the three
# contract conditions share, and each class's own sentence around them —
# spaces included, for they are what its strings hold.

def _operator_str(cond) -> str:
    sb = ">= " if cond.isMore else "<= "
    return f" {sb} {cond.valueToString()}"


def _contract_str(cond) -> str:
    return f"{cond.conId} on {cond.exchange} is {_operator_str(cond)} "


def _price_str(self):
    return (f"{self.TriggerMethodEnum.toStr(self.triggerMethod)} "
            f"price of {_contract_str(self)} ")


def _margin_str(self):
    return f"the margin cushion percent {_operator_str(self)} "


def _time_str(self):
    return f"time is {_operator_str(self)} "


def _execution_str(self):
    # Concatenation, as there: a name nobody stated raises TypeError rather
    # than printing.
    return ("trade occurs for " + self.symbol + " symbol on "
            + self.exchange + " exchange for " + self.secType
            + " security type")


def _percent_change_str(self):
    return f"percent change of {_contract_str(self)} "


def _volume_str(self):
    return f"volume of {_contract_str(self)} "


_STRINGS = {
    "PriceCondition": _price_str,
    "MarginCondition": _margin_str,
    "TimeCondition": _time_str,
    "ExecutionCondition": _execution_str,
    "PercentChangeCondition": _percent_change_str,
    "VolumeCondition": _volume_str,
}


def install(surface) -> None:
    """Claim the six condition classes for the base they answer to."""
    base = surface["OrderCondition"]
    for name in _KINDS:
        base.register(surface[name])

    object_repr = surface["Object"].__repr__
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

    for name in _KINDS:
        cls = surface[name]
        cls.__str__ = _STRINGS[name]
        cls.__repr__ = object_repr
