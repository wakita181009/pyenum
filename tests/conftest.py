"""Session-startup hook that builds the ``pyenum_test`` cdylib via maturin.

The cdylib lives in ``crates/pyenum-test/``. Pytest is launched from the
repository root (see ``pyproject.toml``'s ``testpaths``), so the
``maturin develop`` invocation here references paths relative to that
working directory.

The build happens inside ``pytest_configure``, which runs **before test
collection**. That guarantees a module-level ``import pyenum_test`` in
any test file resolves to the freshly-built cdylib.

If ``maturin`` is unavailable or the build fails, the session exits with
a ``pytest.exit`` carrying a diagnostic message — this is better than a
cryptic ``ImportError`` inside every test module.
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest  # type: ignore[import-not-found]


def _workspace_root() -> Path:
    # conftest.py lives at <repo>/tests/conftest.py
    return Path(__file__).resolve().parent.parent


def _pyenum_test_manifest() -> Path:
    return _workspace_root() / "crates" / "pyenum-test" / "Cargo.toml"


def pytest_configure(config: pytest.Config) -> None:  # noqa: ARG001
    """Rebuild the ``pyenum_test`` cdylib once per test session.

    Runs before collection, so module-level imports of ``pyenum_test`` in
    the test files pick up the freshly-built wheel.
    """
    maturin_path: str | None = shutil.which("maturin")
    if maturin_path is None:
        pytest.exit(
            "maturin is not installed; install it with `uv pip install maturin` "
            "or `pip install maturin` before running the pyenum integration suite.",
            returncode=3,
        )
    assert maturin_path is not None  # narrowed for mypy

    manifest = _pyenum_test_manifest()
    if not manifest.is_file():
        pytest.exit(
            f"pyenum-test crate manifest not found at {manifest}; "
            "run `cargo check --workspace` first.",
            returncode=3,
        )

    cmd: list[str] = [
        maturin_path,
        "develop",
        "--quiet",
        "--manifest-path",
        str(manifest),
    ]
    result = subprocess.run(
        cmd,
        check=False,
        cwd=_workspace_root(),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        pytest.exit(
            "maturin develop failed:\n"
            f"  cmd: {' '.join(cmd)}\n"
            f"  stderr:\n{result.stderr}",
            returncode=3,
        )
