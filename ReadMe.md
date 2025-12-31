# FerrisProof

> *Rust. Verified. Proven.*

FerrisProof is a **full-stack correctness pipeline** for Rust applications, combining **formal modeling (TLA+, Alloy)**, **Rust’s type system**, and **property-based testing** to ensure your systems are **memory-safe, structurally sound, and functionally correct**.

---


## Features

* **Formal Behavioral Modeling**: TLA+ for temporal properties, safety, and concurrency correctness.
* **Structural Invariants**: Alloy models define and validate data constraints.
* **Rust Implementation**: Leverages ownership, type safety, and concurrency guarantees.
* **Property-Based Testing**: `proptest` ensures correctness across a wide range of inputs.
* **Runtime Monitoring**: Optional assertions and logging to enforce invariants during execution.
* **Traceability**: From formal models → Rust implementation → tests → runtime logs.

---

## Architecture Overview

```
+----------------+       +----------------+       +----------------+
|   TLA+ Models  |       |   Alloy Models |       | Rust Modules   |
|  (Temporal)    |       |  (Structural)  |       | Queue, Graph, |
| Safety/Liveness|       |  Constraints   |       | Async Logic   |
+-------+--------+       +-------+--------+       +-------+--------+
        |                        |                        |
        +-----------> Test Input Generation <------------+
                             (scripts/proptest)
                                        |
                                        v
                              +----------------+
                              | Property Tests |
                              +----------------+
                                        |
                                        v
                              +----------------+
                              | Runtime Checks |
                              +----------------+
```

---

## Setup & Installation

1. **Clone the repository**:

```bash
git clone https://github.com/yourusername/ferris-proof.git
cd ferris-proof
```

2. **Install Rust** (if not installed):
   [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

3. **Install TLA+ tools**:
   Download [TLA+ Toolbox](https://lamport.azurewebsites.net/tla/tools.html)

4. **Install Alloy Analyzer**:
   [http://alloytools.org/](http://alloytools.org/)

5. **Install dependencies and run tests**:

```bash
cargo build
cargo test
```

6. **Run property-based tests**:

```bash
cargo test --test property_tests
```

7. **Run integration tests for concurrency scenarios**:

```bash
cargo test --test integration_tests
```

8. **Run model verification scripts**:

```bash
bash scripts/run_model_checks.sh
```

---

## 📂 Project Structure

```
ferris-proof/
├── Cargo.toml             # Rust package file
├── src/                   # Rust implementation
├── models/                # TLA+ & Alloy formal models
├── tests/                 # Property-based and integration tests
├── scripts/               # Automation scripts for test generation and model checks
└── README.md
```

---

## How It Works

1. **Formal Models**: TLA+ defines system behavior; Alloy defines structural invariants.
2. **Rust Implementation**: Core modules are implemented with ownership, types, and async safety.
3. **Property-Based Testing**: Models feed `proptest` generators to automatically verify properties.
4. **Runtime Verification**: Assertions and logging validate invariants during execution.

---

## Future Directions

* Auto-generate Rust property tests from Alloy/TLA+ models.
* Extend support for distributed multi-agent systems.
* Continuous verification in CI/CD pipelines.
* Runtime trace comparison with TLA+ execution paths for advanced correctness monitoring.

---
