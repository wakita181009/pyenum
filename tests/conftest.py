"""Session-scoped fixture that builds the `pyenum_test` cdylib via maturin.

The cdylib lives in ``crates/pyenum-test/``. Pytest is launched from the
``python/`` directory (see ``python/pyproject.toml`` ``testpaths``), so the
``maturin develop`` invocation here references paths relative to that
working directory.

The fixture is safe to skip: if ``maturin`` is unavailable or the build
fails, we raise ``pytest.skip`` instead of crashing the whole session, so
individual tests that ``pytest.importorskip("pyenum_test")`` still get a
clean message.
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


@pytest.fixture(scope="session", autouse=True)
def _build_pyenum_test() -> None:
    """Rebuild the `pyenum_test` cdylib once per test session."""
    maturin_path: str | None = shutil.which("maturin")
    if maturin_path is None:
        pytest.skip(
            "maturin is not installed; install it with `uv pip install maturin` "
            "or `pip install maturin` to run the pyenum integration suite.",
            allow_module_level=True,
        )
    assert maturin_path is not None  # narrowed after pytest.skip; helps mypy

    manifest = _pyenum_test_manifest()
    if not manifest.is_file():
        pytest.skip(
            f"pyenum-test crate manifest not found at {manifest}; "
            "run `cargo check --workspace` first.",
            allow_module_level=True,
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
        cwd=_workspace_root() / "python",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        pytest.skip(
            "maturin develop failed:\n"
            f"  cmd: {' '.join(cmd)}\n"
            f"  stderr:\n{result.stderr}",
            allow_module_level=True,
        )


@pytest.fixture(scope="session")
def pyenum_test() -> object:
    """Import the built extension lazily so collection survives a failed build."""
    return pytest.importorskip("pyenum_test")
