"""A host the wire cannot carry is refused before any connection is tried.

The reference client checks the host before opening anything: a string with
a character outside printable ASCII — a control character or a non-ASCII one —
is reported on the error callback under 579, with the string named, and the
call returns without attempting a connection.
"""

from conftest import NotConnectedProbe
from ib_dx import EClient


def test_a_host_the_wire_cannot_carry_is_refused_before_the_login():
    for host in ("bad\x01host.invalid", "höst.invalid"):
        probe = NotConnectedProbe()
        c = EClient(probe)
        assert c.connect(host, 7497, username="u", password="p") is None
        assert probe.errors == [(-1, 579, f"Invalid symbol in string - {host}")], (
            probe.errors
        )
        assert not c.is_connected()
