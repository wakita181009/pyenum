"""Protocol conformance tests for the ``enum.IntEnum``-backed fixture.

``HttpStatus`` declares explicit integer discriminants (``200 / 404 / 418``)
so value preservation and ``int`` arithmetic interoperability are testable.
"""

from __future__ import annotations

import enum

import pyenum_test


def test_is_intenum_subclass() -> None:
    assert issubclass(pyenum_test.HttpStatus, enum.IntEnum)
    assert issubclass(pyenum_test.HttpStatus, int)


def test_explicit_values_preserved() -> None:
    assert pyenum_test.HttpStatus.Ok.value == 200
    assert pyenum_test.HttpStatus.NotFound.value == 404
    assert pyenum_test.HttpStatus.Teapot.value == 418


def test_lookup_by_value() -> None:
    assert pyenum_test.HttpStatus(200) is pyenum_test.HttpStatus.Ok
    assert pyenum_test.HttpStatus(404) is pyenum_test.HttpStatus.NotFound
    assert pyenum_test.HttpStatus(418) is pyenum_test.HttpStatus.Teapot


def test_integer_comparison() -> None:
    assert pyenum_test.HttpStatus.Ok == 200
    assert pyenum_test.HttpStatus.NotFound != 200
    assert pyenum_test.HttpStatus.Ok < pyenum_test.HttpStatus.NotFound


def test_integer_arithmetic() -> None:
    # IntEnum members decay to plain int under arithmetic.
    assert pyenum_test.HttpStatus.Ok + 0 == 200
    assert int(pyenum_test.HttpStatus.NotFound) == 404


def test_members_in_declaration_order() -> None:
    assert [m.name for m in pyenum_test.HttpStatus] == ["Ok", "NotFound", "Teapot"]
