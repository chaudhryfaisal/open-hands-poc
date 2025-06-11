#!/bin/bash
# Git hooks installation script
# This script installs the custom git hooks for the project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[SETUP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    print_error "Not in a git repository"
    exit 1
fi

print_status "Installing git hooks for Rust project..."

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOKS_DIR="$SCRIPT_DIR"
GIT_HOOKS_DIR=".git/hooks"

# Create git hooks directory if it doesn't exist
mkdir -p "$GIT_HOOKS_DIR"

# Install hooks
hooks=("pre-commit" "pre-push" "commit-msg")

for hook in "${hooks[@]}"; do
    if [ -f "$HOOKS_DIR/$hook" ]; then
        print_status "Installing $hook hook..."
        
        # Backup existing hook if it exists
        if [ -f "$GIT_HOOKS_DIR/$hook" ]; then
            print_warning "Backing up existing $hook hook to $hook.backup"
            mv "$GIT_HOOKS_DIR/$hook" "$GIT_HOOKS_DIR/$hook.backup"
        fi
        
        # Copy and make executable
        cp "$HOOKS_DIR/$hook" "$GIT_HOOKS_DIR/$hook"
        chmod +x "$GIT_HOOKS_DIR/$hook"
        
        print_success "$hook hook installed"
    else
        print_warning "$hook hook not found in $HOOKS_DIR"
    fi
done

# Install additional tools if not present
print_status "Checking for required tools..."

# Check for rustfmt
if ! command -v rustfmt &> /dev/null; then
    print_warning "rustfmt not found. Installing..."
    rustup component add rustfmt
fi

# Check for clippy
if ! command -v cargo-clippy &> /dev/null; then
    print_warning "clippy not found. Installing..."
    rustup component add clippy
fi

# Suggest additional tools
print_status "Suggesting additional tools for better development experience:"

if ! command -v cargo-audit &> /dev/null; then
    print_warning "cargo-audit not found. Install with: cargo install cargo-audit"
fi

if ! command -v cargo-outdated &> /dev/null; then
    print_warning "cargo-outdated not found. Install with: cargo install cargo-outdated"
fi

if ! command -v cargo-watch &> /dev/null; then
    print_warning "cargo-watch not found. Install with: cargo install cargo-watch"
fi

if ! command -v cargo-expand &> /dev/null; then
    print_warning "cargo-expand not found. Install with: cargo install cargo-expand"
fi

# Test the hooks
print_status "Testing hook installation..."

# Test pre-commit hook
if [ -x "$GIT_HOOKS_DIR/pre-commit" ]; then
    print_success "pre-commit hook is executable"
else
    print_error "pre-commit hook is not executable"
fi

# Test pre-push hook
if [ -x "$GIT_HOOKS_DIR/pre-push" ]; then
    print_success "pre-push hook is executable"
else
    print_error "pre-push hook is not executable"
fi

# Test commit-msg hook
if [ -x "$GIT_HOOKS_DIR/commit-msg" ]; then
    print_success "commit-msg hook is executable"
else
    print_error "commit-msg hook is not executable"
fi

print_success "Git hooks installation completed! 🎉"
print_status "Hooks installed:"
echo "  - pre-commit: Runs formatting, linting, and tests before commits"
echo "  - pre-push: Runs comprehensive checks before pushing"
echo "  - commit-msg: Validates commit message format"

print_status "To disable hooks temporarily, use:"
echo "  git commit --no-verify"
echo "  git push --no-verify"

print_status "To uninstall hooks, run:"
echo "  rm .git/hooks/pre-commit .git/hooks/pre-push .git/hooks/commit-msg"