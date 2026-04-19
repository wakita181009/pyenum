"""Protocol conformance tests for the ``enum.StrEnum``-backed fixture.

``Greeting`` uses ``auto()`` — CPython's ``StrEnum._generate_next_value_``
lowercases the member name, so ``Greeting.Hello.value == "hello"``.
"""

from __future__ import annotations

import enum

import pyenum_test


def test_is_strenum_subclass() -> None:
    assert issubclass(pyenum_test.Greeting, enum.StrEnum)
    assert issubclass(pyenum_test.Greeting, str)


def test_auto_value_is_lowercased_name() -> None:
    # StrEnum.auto() generates the lowercased member name as the value.
    assert pyenum_test.Greeting.Hello.value == "hello"
    assert pyenum_test.Greeting.World.value == "world"
    assert pyenum_test.Greeting.Bye.value == "bye"


def test_member_is_str_instance() -> None:
    assert isinstance(pyenum_test.Greeting.Hello, str)


def test_string_equality() -> None:
    # StrEnum members equal their string values.
    assert pyenum_test.Greeting.Hello == "hello"
    assert pyenum_test.Greeting.World != "hello"


def test_string_concatenation() -> None:
    concatenated = pyenum_test.Greeting.Hello + ", World!"
    assert concatenated == "hello, World!"


def test_lookup_by_value() -> None:
    assert pyenum_test.Greeting("hello") is pyenum_test.Greeting.Hello
    assert pyenum_test.Greeting("bye") is pyenum_test.Greeting.Bye


def test_members_in_declaration_order() -> None:
    assert [m.name for m in pyenum_test.Greeting] == ["Hello", "World", "Bye"]
