# natsort-rs

A Rust port of Python's [`natsort`](https://github.com/SethMMorton/natsort) library — natural sorting for strings with embedded numbers.

## 📊 Original Python Test Baseline

The original `SethMMorton/natsort` Python test suite contains **344 test cases**.

| Status | Count |
|--------|-------|
| ✅ Passed | 267 |
| ⏭️ Skipped | 10 |
| ❌ Errored | 67 |
| **Total** | **344** |

Reproduced with `python -m pytest tests/ -v` inside `python_src/.venv` (Python 3.12.3, WSL2/Ubuntu). Full log: `../test_baseline.log`.

> **Note:** None of the 67 errors are library defects — every one is a *collection/setup* error caused by the host environment, not by `natsort` itself:
>
> | Cause | Count | Detail |
> |-------|-------|--------|
> | `locale.Error: unsupported locale setting` | 49 | The `en_US.UTF-8` / `de_DE.UTF-8` locales are not generated in this WSL image, so the `with_locale_*` fixtures in `tests/conftest.py` fail at setup. |
> | `fixture 'mocker' not found` | 18 | `pytest-mock` is not installed; all 18 are CLI tests in `tests/test_main.py`. |
>
> The 10 skips are `de_DE`-locale tests that skip themselves when that locale is absent.
>
> The Rust port targets parity with the **267 passing tests**. Locale-dependent behaviour (`ns.LOCALE`) is still ported, but validated against Python at runtime through the `pyo3` parity harness rather than against these errored tests. See `DECISIONS.md`.

## 🏆 Port Mortem 2026

This project was created for the **Port Mortem 2026** Hackathon (Code Resurrection Wave 2).
```