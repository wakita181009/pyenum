"""Protocol conformance tests for the ``enum.Enum``-backed fixture.

``Color`` is registered with no explicit discriminants, so CPython's
own ``enum.auto()`` assigns ``1, 2, 3`` as values.
"""

from __future__ import annotations

import enum
from typing import Any


def test_is_enum_subclass(pyenum_test: Any) -> None:
    assert issubclass(pyenum_test.Color, enum.Enum)


def test_not_int_or_str_subclass(pyenum_test: Any) -> None:
    assert not issubclass(pyenum_test.Color, enum.IntEnum)
    assert not issubclass(pyenum_test.Color, enum.StrEnum)


def test_members_in_declaration_order(pyenum_test: Any) -> None:
    assert [m.name for m in pyenum_test.Color] == ["Red", "Green", "Blue"]


def test_lookup_by_name(pyenum_test: Any) -> None:
    assert pyenum_test.Color["Red"] is pyenum_test.Color.Red
    assert pyenum_test.Color["Green"] is pyenum_test.Color.Green


def test_lookup_by_value(pyenum_test: Any) -> None:
    assert pyenum_test.Color(1) is pyenum_test.Color.Red
    assert pyenum_test.Color(2) is pyenum_test.Color.Green
    assert pyenum_test.Color(3) is pyenum_test.Color.Blue


def test_auto_values_are_one_based(pyenum_test: Any) -> None:
    assert pyenum_test.Color.Red.value == 1
    assert pyenum_test.Color.Green.value == 2
    assert pyenum_test.Color.Blue.value == 3


def test_member_name_attribute(pyenum_test: Any) -> None:
    assert pyenum_test.Color.Red.name == "Red"
    assert pyenum_test.Color.Blue.name == "Blue"


def test_member_is_instance_of_class(pyenum_test: Any) -> None:
    assert isinstance(pyenum_test.Color.Red, pyenum_test.Color)
    assert isinstance(pyenum_test.Color.Red, enum.Enum)


def test_equality_semantics(pyenum_test: Any) -> None:
    assert pyenum_test.Color.Red == pyenum_test.Color.Red
    assert pyenum_test.Color.Red != pyenum_test.Color.Blue


def test_unknown_value_raises(pyenum_test: Any) -> None:
    import pytest  # type: ignore[import-not-found]

    with pytest.raises(ValueError):
        pyenum_test.Color(999)


def test_unknown_name_raises(pyenum_test: Any) -> None:
    import pytest  # type: ignore[import-not-found]

    with pytest.raises(KeyError):
        _ = pyenum_test.Color["Magenta"]
