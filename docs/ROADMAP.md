# Architectural Roadmap: X32-Rust Evolution

## 📋 Recommended Workplan

| Task | Priority | Complexity | Description |
| :--- | :--- | :--- | :--- |
| **1. Dependency Unification** | Critical | **Small** | Move shared dependencies to root `Cargo.toml` workspace inheritance. |
| **2. Structural Reorganization**| High | **Medium**| Move crates into `libs/`, `apps/`, and `tools/` directories. |
| **3. `MixerClient` Abstraction** | High | **Medium**| Consolidate boilerplate (Heartbeat, Bounding, Bolt) into `x32_lib`. |
| **4. Unified `x32-cli`** | Medium | **Large** | Merge 20+ utility crates into a single subcommand-based binary. |
| **5. Rust-Native Testing** | Medium | **Large** | Port `.sh`/`.ps1` tests to native `assert_cmd` integration tests. |

---

## 🔬 Complexity Analysis & Estimates

### 1. Dependency Unification (Est: 1-2 hours)
*   **Scope**: Editing ~35 `Cargo.toml` files and the root file.
*   **Risk**: Low. Modern Cargo handles this elegantly.
*   **Impact**: Instant build-time improvement.

### 2. Structural Reorganization (Est: 2-4 hours)
*   **Scope**: Physical file moves and updating workspace paths.
*   **Risk**: Medium. Can break internal paths/relative imports if not done surgically.
*   **Impact**: Major improvement to repository discoverability.

### 3. `MixerClient` Abstraction (Est: 1-2 days)
*   **Scope**: Refactoring the core networking logic and updating all binary entry points.
*   **Risk**: High. Changes the fundamental way tools talk to the console.
*   **Impact**: Eliminates 50% of boilerplate and centralizes security/performance fixes.

### 4. Unified `x32-cli` (Est: 3-5 days)
*   **Scope**: Merging entry points and flag logic for dozens of tools.
*   **Risk**: High. Requires careful mapping of legacy CLI arguments to subcommands.
*   **Impact**: Superior UX. One binary to rule them all.

### 5. Rust-Native Testing (Est: Ongoing/Incremental)
*   **Scope**: Rewriting 15+ years of bash/powershell logic into Rust.
*   **Risk**: Low. Can be done tool-by-tool.
*   **Impact**: Guarantees stability across all OS platforms via `cargo test`.

---

## 🤖 Proposed Execution Strategy (Best Practice)

For a refactor of this scale, I recommend the **"Staged Feature Branch"** approach:

1.  **Phase A (The Shell):** I will perform **Task 1 (Dependencies)** and **Task 2 (Folders)** directly to establish the new "Skeleton." This ensures the workspace structure is sound before deep logic changes.
2.  **Phase B (The Core):** I will create a feature branch `refactor/mixer-client` and delegate the implementation of **Task 3** to **Jules AI**. Jules is excellent at exhaustive, repetitive refactoring across many files.
3.  **Phase C (The Consolidation):** Once the core client is stable, we use a second feature branch for **Task 4**, migrating tools into the monolith one by one.

**Best Practice Advice**: Do not attempt all 5 tasks in one PR. A single mistake in the `MixerClient` could break 30 tools simultaneously. By staging the structural changes first, we maintain a buildable state throughout the evolution.

**Should I proceed with the preliminary Skeleton (Tasks 1 & 2) now?**
