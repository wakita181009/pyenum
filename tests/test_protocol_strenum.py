"""Protocol conformance tests for the ``enum.StrEnum``-backed fixture.

``Greeting`` uses ``auto()`` — CPython's ``StrEnum._generate_next_value_``
lowercases the member name, so ``Greeting.Hello.value == "hello"``.

Because ``enum.StrEnum`` only exists on Python 3.11+, this file (along
with ``test_protocol_language.py``) is the single home for every
StrEnum-touching assertion — roundtrip conversion, class-cache identity,
and pickle support included. The module-level ``pytestmark`` is the one
and only 3.10 opt-out point in the test suite.
"""

from __future__ import annotations

import enum
import pickle
import sys

import pytest  # type: ignore[import-not-found]

if sys.version_info < (3, 11):
    # Must short-circuit at collection time: the parametrize decorators
    # below evaluate `pyenum_test.Greeting.*` at import, and pyenum-test's
    # pymodule init skips StrEnum registration on 3.10, so those attributes
    # do not exist. `pytestmark = skipif(...)` would only skip at call time,
    # which is too late to avoid the AttributeError during collection.
    pytest.skip(
        "enum.StrEnum requires Python >= 3.11",
        allow_module_level=True,
    )

import pyenum_test  # noqa: E402


def test_is_strenum_subclass() -> None:
    assert issubclass(pyenum_test.Greeting, enum.StrEnum)
    assert issubclass(pyenum_test.Greeting, str)


def test_color_is_not_strenum_subclass() -> None:
    # Lives here (not in test_protocol_enum.py) because the assertion
    # itself references enum.StrEnum, which only exists on 3.11+.
    assert not issubclass(pyenum_test.Color, enum.StrEnum)


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


# ---------------------------------------------------------------------------
# Cross-cutting tests relocated here so no 3.10 skip markers leak into
# test_conversion.py / test_cache.py / test_pickle.py / test_protocol_enum.py.
# ---------------------------------------------------------------------------


def test_greeting_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.Greeting:
        assert pyenum_test.greeting_roundtrip(variant) is variant


def test_greeting_cache_identity_and_construction_count() -> None:
    assert pyenum_test.Greeting is pyenum_test.Greeting
    assert pyenum_test.Greeting is not pyenum_test.Color
    assert pyenum_test._construction_count(pyenum_test.Greeting) == 1


def test_greeting_class_has_picklable_module_and_qualname() -> None:
    assert pyenum_test.Greeting.__module__ == "pyenum_test"
    assert pyenum_test.Greeting.__qualname__ == "Greeting"


@pytest.mark.parametrize(
    "member",
    [
        pyenum_test.Greeting.Hello,
        pyenum_test.Greeting.World,
        pyenum_test.Greeting.Bye,
    ],
)
def test_greeting_pickle_roundtrip_preserves_identity(member: object) -> None:
    restored = pickle.loads(pickle.dumps(member))
    assert restored is member
