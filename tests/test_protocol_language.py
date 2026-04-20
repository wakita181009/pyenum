"""Protocol conformance tests for a ``StrEnum`` with explicit variant values.

``Language`` uses ``#[pyenum(value = "...")]`` on each variant so the
Python member value is the PascalCase string (``"Rust"``), not the
``auto()``-derived lowercased variant name (``"rust"``).
"""

from __future__ import annotations

import enum
import pickle
import sys

import pytest  # type: ignore[import-not-found]

if sys.version_info < (3, 11):
    # See test_protocol_strenum.py for the rationale — this must skip at
    # collection time, not at call time.
    pytest.skip(
        "enum.StrEnum requires Python >= 3.11",
        allow_module_level=True,
    )

import pyenum_test  # noqa: E402


def test_is_strenum_subclass() -> None:
    assert issubclass(pyenum_test.Language, enum.StrEnum)
    assert issubclass(pyenum_test.Language, str)


def test_explicit_values_preserved() -> None:
    # The #[pyenum(value = "...")] attribute short-circuits auto() —
    # values are the exact strings declared on the Rust side.
    assert pyenum_test.Language.Rust.value == "Rust"
    assert pyenum_test.Language.Python.value == "Python"
    assert pyenum_test.Language.TypeScript.value == "TypeScript"


def test_lookup_by_value() -> None:
    assert pyenum_test.Language("Rust") is pyenum_test.Language.Rust
    assert pyenum_test.Language("Python") is pyenum_test.Language.Python
    assert pyenum_test.Language("TypeScript") is pyenum_test.Language.TypeScript


def test_string_equality() -> None:
    assert pyenum_test.Language.Rust == "Rust"
    assert pyenum_test.Language.Python != "Rust"


def test_members_in_declaration_order() -> None:
    assert [m.name for m in pyenum_test.Language] == ["Rust", "Python", "TypeScript"]


# ---------------------------------------------------------------------------
# Cross-cutting tests relocated here so the StrEnum skip stays isolated.
# ---------------------------------------------------------------------------


def test_language_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.Language:
        assert pyenum_test.language_roundtrip(variant) is variant


def test_language_cache_identity_and_construction_count() -> None:
    assert pyenum_test.Language is pyenum_test.Language
    assert pyenum_test.Language is not pyenum_test.Greeting
    assert pyenum_test._construction_count(pyenum_test.Language) == 1


def test_language_class_has_picklable_module_and_qualname() -> None:
    assert pyenum_test.Language.__module__ == "pyenum_test"
    assert pyenum_test.Language.__qualname__ == "Language"


@pytest.mark.parametrize(
    "member",
    [
        pyenum_test.Language.Rust,
        pyenum_test.Language.Python,
        pyenum_test.Language.TypeScript,
    ],
)
def test_language_pickle_roundtrip_preserves_identity(member: object) -> None:
    restored = pickle.loads(pickle.dumps(member))
    assert restored is member
