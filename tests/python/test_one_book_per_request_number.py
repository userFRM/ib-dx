"""One request number holds one book per mode, and a withdrawal says when it
holds none.

A smart book and a regular book under one number are two books, as a gateway
holds them apart, and both run; the withdrawal takes the book of the mode it
names, and a mode no book was asked in is answered under 310.

Depth is routed by records the engine keeps, so neither surface could see that
a number already held a book: two contracts' rows arrived interleaved under one
number with nothing to tell them apart, the withdrawal named only the later
contract and left the earlier one being served, and a reconnect brought back
one book where there had been two.
"""

import ib_dx


class Errors(ib_dx.EWrapper):
    def __init__(self):
        super().__init__()
        self.seen = []

    def error(self, req_id, error_time, code, msg, advanced_order_reject_json=""):
        self.seen.append((req_id, code, msg))


def contract(con_id, symbol):
    c = ib_dx.Contract()
    c.conId = con_id
    c.symbol = symbol
    c.secType = "STK"
    c.exchange = "SMART"
    c.currency = "USD"
    return c


def test_withdrawing_a_book_that_is_not_held_says_so():
    w = Errors()
    c = ib_dx.EClient(w)
    c._test_connect("T")

    c.cancelMktDepth(7, False)

    c.poll()
    assert w.seen == [
        (7, 310, "Can't find the subscribed market depth with tickerId:7")
    ], f"nothing is held under that number: {w.seen}"


def test_a_second_book_under_a_live_number_is_refused():
    w = Errors()
    c = ib_dx.EClient(w)
    c._test_connect("T")

    c.reqMktDepth(7, contract(756733, "SPY"), 5, False, [])
    assert w.seen == [], f"the first book is asked for: {w.seen}"

    c.reqMktDepth(7, contract(320227571, "QQQ"), 5, False, [])

    c.poll()
    assert w.seen == [
        (7, 322, "Error processing request:-'' : cause - Duplicate ticker id")
    ], f"the number already holds a book: {w.seen}"

    # A smart book and a regular book under one number are two books, as a
    # gateway holds them apart: the second is taken and both run.
    w.seen.clear()
    c.reqMktDepth(7, contract(320227571, "QQQ"), 5, True, [])
    c.poll()
    assert w.seen == [], f"the book of the other mode is taken: {w.seen}"

    # Withdrawn, the mode it names is the caller's again — the smart book
    # runs on, and a second withdrawal of the regular book is answered under
    # 310 rather than taking the other mode's book.
    w.seen.clear()
    c.cancelMktDepth(7, False)
    assert w.seen == [], f"the regular book is withdrawn: {w.seen}"
    c.cancelMktDepth(7, False)
    c.poll()
    assert w.seen == [
        (7, 310, "Can't find the subscribed market depth with tickerId:7")
    ], f"no regular book is held any more: {w.seen}"
    w.seen.clear()
    c.cancelMktDepth(7, True)
    c.poll()
    assert w.seen == [], f"the smart book is withdrawn: {w.seen}"

    # Withdrawn, the number is the caller's again.
    c.reqMktDepth(7, contract(320227571, "QQQ"), 5, False, [])
    assert w.seen == [], f"the number is free again: {w.seen}"
