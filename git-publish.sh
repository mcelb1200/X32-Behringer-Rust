#!/usr/bin/env bash

# Robust git sync and publish helper script
# Designed for mcelb1200/X32-Behringer-Rust

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Ensure we are in a git repository
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    log_error "Not inside a git repository."
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    log_warn "You have uncommitted changes."
    echo "1) Stash changes and proceed"
    echo "2) Commit changes using a caveman-style message"
    echo "3) Abort"
    read -rp "Enter choice [1-3]: " uncommitted_choice
    case $uncommitted_choice in
        1)
            log_info "Stashing changes..."
            git stash
            ;;
        2)
            read -rp "Enter brief commit topic (e.g. 'client refactor'): " commit_topic
            # Convert to caveman style (lowercased, brief, e.g. "refactor: client refactor")
            commit_msg="refactor: $(echo "$commit_topic" | tr '[:upper:]' '[:lower:]')"
            log_info "Committing with message: '${commit_msg}'"
            git commit -am "${commit_msg}"
            ;;
        3)
            log_info "Aborting."
            exit 0
            ;;
        *)
            log_error "Invalid choice."
            exit 1
            ;;
    esac
fi

CURRENT_BRANCH=$(git symbolic-ref --short HEAD)
log_info "Current branch: ${CURRENT_BRANCH}"

# Determine remote (defaulting to origin)
REMOTE="origin"
if ! git remote | grep -q "^${REMOTE}$"; then
    log_warn "Remote '${REMOTE}' not found. Using first available remote."
    REMOTE=$(git remote | head -n 1)
    if [ -z "${REMOTE}" ]; then
        log_error "No remotes configured."
        exit 1
    fi
fi
log_info "Target remote: ${REMOTE}"

# Fetch latest from remote
log_info "Fetching latest changes from ${REMOTE}..."
git fetch "${REMOTE}"

# Check if upstream is configured
UPSTREAM=$(git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || true)

if [ -z "${UPSTREAM}" ]; then
    log_warn "No upstream configured for branch '${CURRENT_BRANCH}'"
    # Check if branch exists on remote
    if git show-ref --verify --quiet "refs/remotes/${REMOTE}/${CURRENT_BRANCH}"; then
        log_info "Remote branch exists. Setting upstream tracking..."
        git branch --set-upstream-to="${REMOTE}/${CURRENT_BRANCH}" "${CURRENT_BRANCH}"
        UPSTREAM="${REMOTE}/${CURRENT_BRANCH}"
    else
        log_info "Remote branch does not exist yet."
    fi
fi

# Compare local and remote
if [ -n "${UPSTREAM}" ]; then
    LOCAL_HASH=$(git rev-parse HEAD)
    REMOTE_HASH=$(git rev-parse "${UPSTREAM}")
    BASE_HASH=$(git merge-base HEAD "${UPSTREAM}")

    if [ "${LOCAL_HASH}" = "${REMOTE_HASH}" ]; then
        log_success "Branch is up-to-date with remote."
        exit 0
    elif [ "${LOCAL_HASH}" = "${BASE_HASH}" ]; then
        log_warn "Local branch is BEHIND remote. Fast-forwarding..."
        git merge --ff-only "${UPSTREAM}"
        log_success "Successfully fast-forwarded to latest remote state."
        exit 0
    elif [ "${REMOTE_HASH}" = "${BASE_HASH}" ]; then
        log_info "Local branch is AHEAD of remote. Pushing..."
        git push "${REMOTE}" "${CURRENT_BRANCH}"
        log_success "Pushed local commits successfully."
        exit 0
    else
        log_warn "Local and remote branches have DIVERGED."
        echo "Please select an integration strategy:"
        echo "1) Rebase local changes on top of remote (Clean history - Recommended)"
        echo "2) Merge remote changes into local (Creates merge commit)"
        echo "3) Force push local changes (Safely overwrite remote - WARNING: Overwrites remote history)"
        echo "4) Abort and backup local branch"
        read -rp "Enter choice [1-4]: " choice

        case $choice in
            1)
                log_info "Rebasing local changes on top of ${UPSTREAM}..."
                if git rebase "${UPSTREAM}"; then
                    log_success "Rebase successful. Pushing..."
                    git push "${REMOTE}" "${CURRENT_BRANCH}"
                    log_success "Published successfully."
                else
                    log_error "Rebase failed. Resolve conflicts manually and run 'git rebase --continue'."
                    exit 1
                fi
                ;;
            2)
                log_info "Merging ${UPSTREAM} into local..."
                if git merge "${UPSTREAM}"; then
                    log_success "Merge successful. Pushing..."
                    git push "${REMOTE}" "${CURRENT_BRANCH}"
                    log_success "Published successfully."
                else
                    log_error "Merge failed. Resolve conflicts manually and commit."
                    exit 1
                fi
                ;;
            3)
                log_warn "Force pushing to ${REMOTE}/${CURRENT_BRANCH} using --force-with-lease..."
                if git push --force-with-lease "${REMOTE}" "${CURRENT_BRANCH}"; then
                    log_success "Force pushed successfully."
                else
                    log_error "Force push rejected. Remote has changes you do not have."
                    exit 1
                fi
                ;;
            4)
                BACKUP_BRANCH="${CURRENT_BRANCH}-backup-$(date +%s)"
                log_info "Creating backup branch ${BACKUP_BRANCH}..."
                git branch "${BACKUP_BRANCH}"
                log_success "Backup created. Aborting."
                exit 0
                ;;
            *)
                log_error "Invalid choice."
                exit 1
                ;;
        esac
    fi
else
    # Upstream not configured and doesn't exist on remote
    log_info "Pushing new branch ${CURRENT_BRANCH} to ${REMOTE} and setting upstream tracking..."
    git push -u "${REMOTE}" "${CURRENT_BRANCH}"
    log_success "Successfully published branch."
fi
