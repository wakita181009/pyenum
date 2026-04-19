"""Protocol conformance tests for a ``StrEnum`` with explicit variant values.

``Language`` uses ``#[pyenum(value = "...")]`` on each variant so the
Python member value is the PascalCase string (``"Rust"``), not the
``auto()``-derived lowercased variant name (``"rust"``).
"""

from __future__ import annotations

import enum

import pyenum_test


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
