"""A condition prints the sentence the reference client writes.

Printing a condition here answered an invented repr —
`PriceCondition(conId=None, price < None)` — instead of the reference's
chained condition text (`Default price of 756733 on SMART is  >=  200.0  `),
so a log scraper or a test matching that text found nothing it recognises,
and the conjunction wording the base writes was absent. Each class now
renders its own sentence through the shared operator and contract wordings,
exactly as the reference chains them, and repr is the Object base's — the
id, a colon and that same text. The expected strings below are what the
reference client's own classes print for the same field values.

Run: pytest tests/python/test_a_condition_prints_the_text_the_reference_writes.py -v
"""

import pytest

from ibkr_dx import (
    ExecutionCondition, MarginCondition, PercentChangeCondition,
    PriceCondition, TimeCondition, VolumeCondition,
)


def test_each_condition_prints_the_reference_sentence():
    for cond, text in [
        (PriceCondition(0, 756733, "SMART", True, 200.0),
         "Default price of 756733 on SMART is  >=  200.0  "),
        (PriceCondition(2, 756733, "SMART", False, 195.5),
         "Last price of 756733 on SMART is  <=  195.5  "),
        # A method nobody stated reads NOTFOUND, as the enum answers there.
        (PriceCondition(),
         "NOTFOUND price of None on None is  <=  None  "),
        (TimeCondition(False, "20260101 09:30:00"),
         "time is  <=  20260101 09:30:00 "),
        (TimeCondition(), "time is  <=  None "),
        (MarginCondition(True, 25), "the margin cushion percent  >=  25 "),
        (MarginCondition(), "the margin cushion percent  <=  None "),
        (ExecutionCondition("STK", "SMART", "AAPL"),
         "trade occurs for AAPL symbol on SMART exchange for STK security type"),
        (VolumeCondition(756733, "SMART", True, 1_000_000),
         "volume of 756733 on SMART is  >=  1000000  "),
        (PercentChangeCondition(756733, "SMART", False, 5.0),
         "percent change of 756733 on SMART is  <=  5.0  "),
        (PercentChangeCondition(),
         "percent change of None on None is  <=  1.7976931348623157e+308  "),
    ]:
        assert str(cond) == text, type(cond).__name__
        # And repr is the Object base's: the id, a colon, that text.
        assert repr(cond) == f"{id(cond)}: {text}", type(cond).__name__


def test_an_execution_condition_missing_a_name_raises_as_the_reference_does():
    # There the sentence is built by concatenation, so a name nobody stated
    # raises TypeError rather than printing.
    with pytest.raises(TypeError):
        str(ExecutionCondition())
