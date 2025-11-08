# CI and Branching Strategy

This document outlines the Continuous Integration (CI) setup and the recommended branching strategy for this repository.

## Continuous Integration (CI)

The CI pipeline is managed by GitHub Actions and is defined in the workflow file located at `.github/workflows/build.yml`.

### Workflow Triggers

The CI workflow is automatically triggered on the following events:

-   **Push:** Every time a commit is pushed to the `main` branch.
-   **Pull Request:** Every time a pull request is opened or updated that targets the `main` branch.

### Build Matrix

To ensure cross-platform compatibility, the workflow runs a matrix of jobs across different operating systems and target architectures.

**Desktop Platforms (Build & Test):**
The following platforms are fully built and tested:
-   Ubuntu Linux (`x86_64-unknown-linux-gnu`)
-   Windows (`x86_64-pc-windows-msvc`)
-   macOS (Intel) (`x86_64-apple-darwin`)
-   macOS (Apple Silicon) (`aarch64-apple-darwin`)

**Mobile Platforms (Build Only):**
The following platforms are compiled to ensure the code builds, but tests are not run:
-   Android (`aarch64-linux-android`)
-   iOS (`aarch64-apple-ios`)

### Job Steps

Each job in the build matrix performs the following steps:
1.  **Checkout:** Checks out the repository's source code.
2.  **Install Rust:** Installs the stable Rust toolchain for the specific target architecture.
3.  **Build:** Compiles the entire workspace in release mode for the specified target.
4.  **Test:** Runs the full test suite (`cargo test --workspace --release`) for the desktop platform targets.

## Branch Protection Strategy

To maintain the stability and quality of the `main` branch, it is highly recommended to configure branch protection rules on GitHub. These rules prevent direct pushes to `main` and ensure that all code is reviewed and passes CI checks before being merged.

### Recommended Rules for the `main` branch:

1.  **Require a pull request before merging**
    -   Enable this to force all changes to go through a pull request review process.

2.  **Require status checks to pass before merging**
    -   Enable this to ensure that all CI jobs in the build matrix must pass before a pull request can be merged.
    -   You should require the following status checks:
        -   `Build on ubuntu-latest for x86_64-unknown-linux-gnu`
        -   `Build on windows-latest for x86_64-pc-windows-msvc`
        -   `Build on macos-latest for x86_64-apple-darwin`
        -   `Build on macos-latest for aarch64-apple-darwin`
        -   (And the mobile build jobs if desired)

3.  **Require conversation resolution before merging**
    -   A useful rule to ensure all review comments are addressed before merging.

4.  **Require linear history**
    -   Prevents merge commits and keeps the project history clean and easy to follow.

You can configure these rules in your repository's settings under **Branches** > **Add branch protection rule**.
