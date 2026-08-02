# Master Implementation Prompt

## Objective
You are a **Senior Rust Engineer**. Your task is to port the Python library `SethMMorton/natsort` to idiomatic, high-performance Rust, ensuring 100% behavioral parity with the original library.

## Directory Structure
The working directory is `/mnt/d/workspace/port_mortem/`. The layout is:
```
/mnt/d/workspace/port_mortem/
|--- natsort-rs/          # This repo (you are here)
│   |--- .agent/          # Planning docs
│   |--- src/
│   |--- tests/
│   |--- README.md
|--- python_src/          # Original Python repo (sibling folder)
|--- .hashes/             # Hash files (sibling folder)
```

**Important:** `python_src` and `.hashes` are **sibling folders**, not inside `natsort-rs`. Use `../python_src` and `../.hashes` to access them.

## Context & Resources
All planning documents are located in the `.agent/` folder. You must read and strictly follow these files:
1.  **`.agent/plan.md`**: The step-by-step execution roadmap and timeline.
2.  **`.agent/architecture.md`**: High-level design, module map, data flow, and parity strategy.
3.  **`.agent/samples.md`**: Concrete code comparisons (Python vs. Rust) and usage examples.
4.  **`.agent/readme_guide.md`**: Instructions for constructing the final `README.md`.
5.  **`.agent/understanding_algo.md`**: Deep dive into the natsort algorithm.
6.  **`../python_src/`**: The original Python repository (sibling folder).

## Workflow Rules
1.  **Phase-by-Phase Execution**: Execute one phase at a time. Do not rush.
2.  **Parity is King**: You must implement a Python bridge (`pyo3`) to compare Rust outputs against Python outputs for every feature.
3.  **Honest Documentation**: If you cannot port something 1:1, or if you make a non-obvious architectural choice, you **MUST** update `DECISIONS.md` immediately with the format specified in `plan.md`.
4.  **Maintainability**: Code must be clean, well-documented, and idiomatic Rust. Use `thiserror` for errors, `regex` for parsing, and `criterion` for benchmarks.
5.  **Commits**: Stop at every `[COMMIT CHECKPOINT]` and prompt the user to run the exact `git commit` command provided.
6.  **Architecture Adherence**: Strictly follow the module structure defined in `architecture.md`. Do not create ad-hoc files unless necessary for a specific feature.

## Execution Steps

### Phase 0: Repository Initialization & Baseline
1.  **Verify Prerequisites**: Ensure `../python_src` exists and contains the original natsort source.
2.  **Hash Original Tests**: Capture the integrity hash of the original test files:
    ```bash
    mkdir -p ../.hashes
    sha256sum ../python_src/tests/*.py > ../.hashes/kickoff_tests_1.sha256
    ```
3.  **Run Python Test Suite**: Execute the original Python tests to establish a baseline:
    ```bash
    cd ../python_src
    source .venv/bin/activate
    pip install -e .
    pip install pytest
    python -m pytest tests/ -v 2>&1 | tee ../test_baseline.log
    deactivate
    ```
4.  **Record Results**: The original test suite has **344 test cases**. Record the exact results:
    - Passed: **267**
    - Skipped: **10**
    - Errored: **67**
5.  **Update README.md**: Create the initial `README.md` if not already presetn else upate it with a baseline section:
    ```markdown
    ## 📊 Original Python Test Baseline

    The original `SethMMorton/natsort` Python test suite contains **344 test cases**.

    | Status | Count |
    |--------|-------|
    | ✅ Passed | 267 |
    | ⏭️ Skipped | 10 |
    | ❌ Errored | 67 |
    | **Total** | **344** |

    > **Note:** The 67 errored tests are due to platform-specific issues and deprecated test fixtures in the original Python repository. The Rust port will focus on achieving parity with the 267 passing tests.
    ```
6.  **Verify pyo3 Setup**: Ensure `pyo3` is in `Cargo.toml` dev-dependencies and Python is discoverable:
    ```bash
    python3 --version  # Should work from WSL
    # pyo3 will use this Python for the parity harness
    ```
7.  **Create Parity Harness Scaffold**: Create `tests/parity.rs` with the initial pyo3 bridge:
    ```rust
    use pyo3::prelude::*;

    #[test]
    fn test_pyimport_works() {
        Python::with_gil(|py| {
            let natsort = py.import("natsort").expect("Failed to import natsort");
            let version: String = natsort.getattr("__version__").unwrap().extract().unwrap();
            println!("natsort version: {}", version);
        });
    }
    ```
    Run `cargo test test_pyimport_works` to verify pyo3 can find and import the Python natsort module.
8.  **STOP** and prompt for Commit Checkpoint 0.

    **Commit Command:**
    ```bash
    git add .agent/ .gitignore README.md Cargo.toml src/
    git commit -m "Phase 0: Initial commit with planning docs and baseline"
    ```

### Phase 1: Architecture & Core Parity
1.  Follow `.agent/plan.md` Phase 1.
2.  Reference `.agent/architecture.md` for module structure (`segment.rs`, `ns.rs`, etc.).
3.  Reference `.agent/samples.md` for API design (e.g., `natsorted` signature).
4.  **Parity Harness**: Expand `tests/parity.rs` to compare Rust vs Python outputs:
    - Use `pyo3::Python::with_gil` to import `natsort` and call `natsort.natsorted(input)`
    - Call your Rust `natsort::natsorted(&input)`
    - Assert both outputs are identical
    - For each new feature, add a parity test before implementing it
5.  **STOP** and prompt for Commit Checkpoint 1.

### Phase 2: Flags & Mixed Types
1.  Follow `.agent/plan.md` Phase 2.
2.  Implement `NsFlags` (REAL, IGNORECASE, etc.) in `src/ns.rs`.
3.  Implement `Ord` for `Segment` in `src/mixed.rs` (cross-type comparison).
4.  Handle Mixed Types and Recursive Descent.
5.  **STOP** and prompt for Commit Checkpoint 2.

### Phase 3: Advanced Features & Fuzzing
1.  Follow `.agent/plan.md` Phase 3.
2.  Implement `os_sorted()` and Bytes handling in `src/os_sort.rs`.
3.  Build the Differential Fuzz Harness (`cargo-fuzz`).
4.  Update `DECISIONS.md` for any divergences found.
5.  **STOP** and prompt for Commit Checkpoint 3.

### Phase 4: Benchmarks & Polish
1.  Follow `.agent/plan.md` Phase 4.
2.  Run `criterion` benchmarks and generate `benchmarks.md`.
3.  Update `README.md` using `.agent/readme_guide.md`
4.  **STOP** and prompt for Commit Checkpoint 4.

### Phase 5: Demo & Submission
1.  Follow `.agent/plan.md` Phase 5.
2.  Finalize `DECISIONS.md` and all deliverables.
3.  **STOP** and prompt for Commit Checkpoint 5.

## 🚀 Start Command
**User will say:** "Start the natsort port."
**You will:**
1.  Verify `../python_src` exists.
2.  Read `.agent/architecture.md`, `.agent/samples.md`, and `.agent/readme_guide.md`.
3.  Begin Phase 0 execution.
