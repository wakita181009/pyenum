"""Cache identity tests.

The ``PyEnum`` derive installs a per-type, per-interpreter ``PyOnceLock``
that holds the Python class object. The class identity MUST be stable
across repeated lookups and across the conversion boundary — two calls
that resolve the same Rust enum type yield the same Python object.
"""

from __future__ import annotations

from typing import Any


def test_class_identity_stable_across_repeated_access(pyenum_test: Any) -> None:
    first = pyenum_test.Color
    second = pyenum_test.Color
    assert first is second


def test_class_identity_stable_across_conversion_boundary(pyenum_test: Any) -> None:
    # Call the roundtrip #[pyfunction] many times; the member object
    # returned each time should be the same Python object as the class
    # attribute, and the class attribute itself should remain stable.
    cls_before = pyenum_test.Color
    members = [
        pyenum_test.color_roundtrip(pyenum_test.Color.Red) for _ in range(10_000)
    ]
    cls_after = pyenum_test.Color

    assert cls_before is cls_after
    for member in members:
        assert member is pyenum_test.Color.Red


def test_each_derived_type_gets_its_own_class(pyenum_test: Any) -> None:
    # Distinct Rust enums must map to distinct Python classes, even when
    # they use the same base.
    assert pyenum_test.Permission is not pyenum_test.BitPerms
    assert pyenum_test.Color is not pyenum_test.Greeting


def test_member_identity_stable_for_each_variant(pyenum_test: Any) -> None:
    # `is` identity for every round-trip ensures the cache returns the
    # same member object rather than a fresh clone.
    for variant in pyenum_test.HttpStatus:
        for _ in range(100):
            assert pyenum_test.http_roundtrip(variant) is variant
