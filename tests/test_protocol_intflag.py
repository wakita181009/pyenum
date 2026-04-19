"""Protocol conformance tests for the ``enum.IntFlag``-backed fixture.

``BitPerms`` is ``Read | Write | Execute | Admin`` with explicit
powers-of-two values (1, 2, 4, 8). ``IntFlag`` combines ``Flag`` semantics
with ``int`` behavior — members can be used wherever an ``int`` is expected.
"""

from __future__ import annotations

import enum

import pyenum_test


def test_is_intflag_subclass() -> None:
    assert issubclass(pyenum_test.BitPerms, enum.IntFlag)
    assert issubclass(pyenum_test.BitPerms, int)


def test_explicit_values_preserved() -> None:
    assert pyenum_test.BitPerms.Read.value == 1
    assert pyenum_test.BitPerms.Write.value == 2
    assert pyenum_test.BitPerms.Execute.value == 4
    assert pyenum_test.BitPerms.Admin.value == 8


def test_bitwise_or_combines_values() -> None:
    admin_rw = (
        pyenum_test.BitPerms.Admin
        | pyenum_test.BitPerms.Read
        | pyenum_test.BitPerms.Write
    )
    assert int(admin_rw) == 11


def test_integer_comparison() -> None:
    assert pyenum_test.BitPerms.Read == 1
    assert pyenum_test.BitPerms.Write == 2
    assert pyenum_test.BitPerms.Admin == 8


def test_integer_arithmetic() -> None:
    assert pyenum_test.BitPerms.Read + pyenum_test.BitPerms.Write == 3


def test_bitwise_and_isolates_bits() -> None:
    combined = pyenum_test.BitPerms.Read | pyenum_test.BitPerms.Admin
    assert (combined & pyenum_test.BitPerms.Read) is pyenum_test.BitPerms.Read
    assert (combined & pyenum_test.BitPerms.Admin) is pyenum_test.BitPerms.Admin


def test_membership_in_combined() -> None:
    combined = pyenum_test.BitPerms.Read | pyenum_test.BitPerms.Execute
    assert pyenum_test.BitPerms.Read in combined
    assert pyenum_test.BitPerms.Execute in combined
    assert pyenum_test.BitPerms.Write not in combined
