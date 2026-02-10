# Contributing

This page covers how to set up a development environment for rapid_textrank and, for maintainers, how to publish releases.

## Development Setup

### Prerequisites

- **Rust 1.70+** -- install from [rustup.rs](https://rustup.rs/)
- **Python 3.9+** -- install from [python.org](https://www.python.org/downloads/)
- **maturin** -- the Rust/Python build tool (`pip install maturin`)

### Building from Source

```bash
git clone https://github.com/xang1234/rapid-textrank
cd rapid_textrank
pip install maturin
maturin develop --release
```

### Installing Dev Dependencies

```bash
pip install -e ".[dev]"
```

This installs the package in editable mode along with development dependencies (pytest, etc.).

### Running Python Tests

```bash
pytest
```

### Running Rust Tests

```bash
cargo test
```

This runs the full Rust test suite, including unit tests and integration tests.

## For Maintainers

### Publishing

Publishing is automated with GitHub Actions using **Trusted Publishing (OIDC)**, so no API tokens need to be stored as secrets.

#### TestPyPI Release

Push a tag matching the `test-*` pattern:

```bash
git tag -a test-0.1.0 -m "TestPyPI 0.1.0"
git push origin test-0.1.0
```

This triggers the `.github/workflows/publish-testpypi.yml` workflow.

#### PyPI Release

Push a tag matching the `v*` pattern:

```bash
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin v0.1.0
```

This triggers the `.github/workflows/publish-pypi.yml` workflow.

#### Wheel Builds

GitHub Actions builds wheels for:

- **Python versions:** 3.9, 3.10, 3.11, 3.12
- **Platforms:** Linux (manylinux), macOS (x86_64, arm64), Windows (x86_64)

#### Trusted Publisher Setup

Before the first publish, add Trusted Publishers on both TestPyPI and PyPI:

- **Repository:** `xang1234/textranker`
- **Workflows:**
    - `.github/workflows/publish-testpypi.yml`
    - `.github/workflows/publish-pypi.yml`
- **Environments:**
    - `testpypi`
    - `pypi`

You can also trigger either workflow manually via the GitHub Actions UI if needed.
