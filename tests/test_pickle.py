"""Pickle round-trip coverage for ``#[derive(PyEnum)]`` fixtures.

The derive forwards ``#[pyenum(module = "...")]`` to CPython's functional
``enum.*`` constructor as the ``module=`` kwarg. Without it, CPython's
``_make_class_unpicklable`` installs a ``__reduce_ex__`` that raises
``TypeError``; with it, ``pickle.dumps(member)`` + ``pickle.loads`` returns
the same ``is``-identical member object.

Each positive case asserts:

1. ``pickle.dumps`` succeeds.
2. ``pickle.loads`` returns an object ``is``-identical to the original
   (enum member identity is preserved across the round trip).

The negative case asserts that an enum *without* the attribute fails at
``pickle.dumps`` with ``TypeError`` — locking in the documented
"opt-in for pickle" behaviour.
"""

from __future__ import annotations

import pickle

import pytest  # type: ignore[import-not-found]

import pyenum_test


@pytest.mark.parametrize(
    "member",
    [
        pyenum_test.Color.Red,
        pyenum_test.Color.Green,
        pyenum_test.Color.Blue,
        pyenum_test.HttpStatus.Ok,
        pyenum_test.HttpStatus.NotFound,
        pyenum_test.HttpStatus.Teapot,
        pyenum_test.Greeting.Hello,
        pyenum_test.Greeting.World,
        pyenum_test.Greeting.Bye,
        pyenum_test.Language.Rust,
        pyenum_test.Language.Python,
        pyenum_test.Language.TypeScript,
        pyenum_test.Permission.Read,
        pyenum_test.Permission.Write,
        pyenum_test.Permission.Execute,
        pyenum_test.BitPerms.Read,
        pyenum_test.BitPerms.Write,
        pyenum_test.BitPerms.Execute,
        pyenum_test.BitPerms.Admin,
    ],
)
def test_member_pickle_roundtrip_preserves_identity(member: object) -> None:
    data = pickle.dumps(member)
    restored = pickle.loads(data)
    assert restored is member


@pytest.mark.parametrize(
    "composite",
    [
        pyenum_test.Permission.Read | pyenum_test.Permission.Write,
        pyenum_test.BitPerms.Read | pyenum_test.BitPerms.Execute,
    ],
)
def test_flag_composite_pickle_roundtrip_preserves_equality(composite: object) -> None:
    restored = pickle.loads(pickle.dumps(composite))
    assert restored == composite


def test_class_module_and_qualname_are_set_for_picklable_classes() -> None:
    for cls in (
        pyenum_test.Color,
        pyenum_test.HttpStatus,
        pyenum_test.Greeting,
        pyenum_test.Language,
        pyenum_test.Permission,
        pyenum_test.BitPerms,
    ):
        assert cls.__module__ == "pyenum_test"
        assert cls.__qualname__ == cls.__name__


def test_enum_without_module_attribute_is_unpicklable() -> None:
    # CPython may raise either `TypeError` (when `_make_class_unpicklable`
    # replaces `__reduce_ex__`) or `pickle.PicklingError` (when frame-walking
    # resolved some unrelated module name and pickle fails at attribute
    # lookup). Both outcomes prove the class is not safely picklable.
    with pytest.raises((TypeError, pickle.PicklingError)):
        pickle.dumps(pyenum_test.UnpicklableColor.Red)
