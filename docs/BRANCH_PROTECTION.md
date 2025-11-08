# Branch Protection Rules for the `master` Branch

To maintain the stability and quality of our codebase, the `master` branch is protected by a set of rules. All contributions to the Rust code must be in idiomatic Rust, and all code should be platform-agnostic unless a specific justification is provided. This document outlines the branch protection rules, their purpose, and the process for contributing.

## 1. Require a Pull Request Before Merging

**Rule:** All changes must be made through a pull request (PR). Direct pushes to the `master` branch are disabled.

**Purpose:** This ensures that every change to the `master` branch is reviewed and verified before being merged. It creates an opportunity for collaboration and code review, which is essential for maintaining code quality.

**Justification for Departure:** There are virtually no exceptions to this rule. In the rare case of a critical hotfix that needs to be deployed immediately, the change should still go through an expedited PR process.

## 2. Require Status Checks to Pass Before Merging

**Rule:** The strict CI pipeline must run and pass on all pull requests before they can be merged.

**Purpose:** This rule automates the process of checking for common errors, enforcing code style, and ensuring that the code compiles and runs on all supported platforms (Linux, macOS, and Windows). This prevents regressions and helps maintain a high standard of code quality.

**The CI pipeline includes the following checks:**
*   `cargo check`: Verifies that the code compiles.
*   `cargo fmt --check`: Ensures that the code adheres to standard Rust formatting.
*   `cargo clippy -- -D warnings`: Lints the code for common mistakes and un-idiomatic patterns, treating all warnings as errors.
*   `cargo test --workspace`: Runs all tests in the workspace to ensure that existing functionality is not broken.
*   `cargo doc`: Verifies that the documentation builds correctly.

**Justification for Departure:** A status check should only be bypassed if it is failing for reasons unrelated to the PR's changes (e.g., a temporary infrastructure issue). In such cases, a project maintainer must investigate the failure and manually approve the merge.

## 3. Require at Least One Code Review Approval

**Rule:** Every pull request must be reviewed and approved by at least one other contributor before it can be merged.

**Purpose:** Code reviews are a critical part of the development process. They help catch bugs, improve code quality, and share knowledge among the team. A second pair of eyes can often spot issues that the original author may have missed.

**Justification for Departure:** This rule should not be bypassed. Even for minor changes, a review is necessary to ensure that the change is correct and does not have any unintended side effects.

## 4. Dismiss Stale Approvals When New Commits Are Pushed

**Rule:** When new commits are pushed to a pull request, any existing approvals are dismissed. A new review is required.

**Purpose:** This ensures that reviewers are always looking at the most up-to-date version of the code. It prevents a situation where a PR is approved, then changed in a way that introduces a bug, and then merged without a second review.

**Justification for Departure:** There are no exceptions to this rule. If a change is made to a PR after it has been approved, it must be reviewed again.

## Common Contributor Scenarios

*   **New Feature:** If you are adding a new feature, please create a new branch from `master`, make your changes, and then open a pull request. Make sure to include tests for your new feature.
*   **Bug Fix:** If you are fixing a bug, please follow the same process as for a new feature. Include a test that reproduces the bug and verifies that your fix works.
*   **Documentation:** For changes to documentation, a pull request is still required. This allows for a review of the clarity and accuracy of the documentation.

By following these rules, we can ensure that our project remains stable, reliable, and a pleasure to work on for everyone.

## Automating Branch Protection with the GitHub API

You can automate the application of these branch protection rules using the GitHub REST API. This is useful for setting up new repositories with a consistent ruleset.

### JSON Ruleset

The following JSON payload represents the branch protection rules described in this document. You can also find this in the `branch_protection.json` file in the root of this repository.

```json
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "build (ubuntu-latest)",
      "build (macos-latest)",
      "build (windows-latest)"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false
}
```

### Applying the Ruleset with `curl`

You can apply these rules to the `master` branch of your repository using the following `curl` command. Make sure to replace `YOUR_TOKEN`, `OWNER`, and `REPO` with your personal access token, the repository owner, and the repository name, respectively.

```bash
curl -L \
  -X PUT \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/branches/master/protection \
  -d @branch_protection.json
```
