"""Protocol conformance tests for the ``enum.Flag``-backed fixture.

``Permission`` declares explicit powers-of-two so bitwise composition has
well-defined values.
"""

from __future__ import annotations

import enum

import pyenum_test


def test_is_flag_subclass() -> None:
    assert issubclass(pyenum_test.Permission, enum.Flag)
    # Plain Flag is NOT a subclass of int (unlike IntFlag).
    assert not issubclass(pyenum_test.Permission, int)


def test_explicit_values_preserved() -> None:
    assert pyenum_test.Permission.Read.value == 1
    assert pyenum_test.Permission.Write.value == 2
    assert pyenum_test.Permission.Execute.value == 4


def test_bitwise_or_produces_combined_member() -> None:
    rw = pyenum_test.Permission.Read | pyenum_test.Permission.Write
    assert rw.value == 3


def test_bitwise_and_isolates_bits() -> None:
    rw = pyenum_test.Permission.Read | pyenum_test.Permission.Write
    assert (rw & pyenum_test.Permission.Read) is pyenum_test.Permission.Read


def test_membership_in_combined() -> None:
    rw = pyenum_test.Permission.Read | pyenum_test.Permission.Write
    assert pyenum_test.Permission.Read in rw
    assert pyenum_test.Permission.Write in rw
    assert pyenum_test.Permission.Execute not in rw


def test_xor_cancels() -> None:
    rw = pyenum_test.Permission.Read | pyenum_test.Permission.Write
    assert (rw ^ pyenum_test.Permission.Read) is pyenum_test.Permission.Write


def test_canonical_member_iteration() -> None:
    # Iteration yields only canonical (non-composite) members in order.
    assert [m.name for m in pyenum_test.Permission] == ["Read", "Write", "Execute"]
