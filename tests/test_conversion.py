"""Bidirectional conversion tests.

Every derived enum exposes a ``*_roundtrip`` ``#[pyfunction]`` that takes
and returns the enum type. The Rust side extracts the variant via
``FromPyObject`` and re-emits it via ``IntoPyObject``, so these tests prove
both halves of the conversion boundary.
"""

from __future__ import annotations

import pyenum_test
import pytest  # type: ignore[import-not-found]


def test_color_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.Color:
        assert pyenum_test.color_roundtrip(variant) is variant


def test_http_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.HttpStatus:
        assert pyenum_test.http_roundtrip(variant) is variant


def test_greeting_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.Greeting:
        assert pyenum_test.greeting_roundtrip(variant) is variant


def test_language_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.Language:
        assert pyenum_test.language_roundtrip(variant) is variant


def test_permission_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.Permission:
        assert pyenum_test.permission_roundtrip(variant) is variant


def test_bitperms_roundtrip_preserves_identity() -> None:
    for variant in pyenum_test.BitPerms:
        assert pyenum_test.bitperms_roundtrip(variant) is variant


def test_foreign_object_raises_type_error() -> None:
    with pytest.raises(TypeError, match="Color"):
        pyenum_test.color_roundtrip(42)  # type: ignore[arg-type]


def test_wrong_enum_type_raises_type_error() -> None:
    # Passing an HttpStatus member into a function expecting Color must
    # fail with TypeError rather than coerce or crash.
    with pytest.raises(TypeError, match="Color"):
        pyenum_test.color_roundtrip(pyenum_test.HttpStatus.Ok)  # type: ignore[arg-type]


def test_none_raises_type_error() -> None:
    with pytest.raises(TypeError):
        pyenum_test.color_roundtrip(None)  # type: ignore[arg-type]
