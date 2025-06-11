# Git Hooks for Rust Reverse Shell Project

This directory contains custom git hooks to ensure code quality and consistency across the project.

## Available Hooks

### 1. Pre-commit Hook (`pre-commit`)
Runs before each commit to ensure code quality:

- **Code Formatting**: Runs `cargo fmt` to ensure consistent code formatting
- **Linting**: Runs `cargo clippy` with strict warnings (`-D warnings`)
- **Tests**: Executes the full test suite to catch regressions
- **Security Scan**: Runs `cargo audit` if available
- **Code Analysis**: Checks for:
  - TODO/FIXME comments in staged changes
  - Debug print statements (`println!`, `dbg!`, `eprintln!`)
  - Large files (>1MB)
  - Invalid Cargo.toml files

### 2. Pre-push Hook (`pre-push`)
Runs before pushing to remote repositories:

- **Comprehensive Testing**: Runs tests in release mode
- **Release Mode Linting**: Runs clippy in release mode
- **Documentation**: Verifies documentation builds correctly
- **Security Audit**: Comprehensive security vulnerability scan
- **Binary Size Check**: Warns about large binaries (>50MB)
- **Sensitive Information**: Scans for potential secrets or credentials
- **Commit Message Validation**: Checks commit message quality
- **Branch Protection**: Prevents accidental force pushes to main/master

### 3. Commit Message Hook (`commit-msg`)
Validates commit message format and content:

- **Length Validation**: Ensures messages are between 10-72 characters
- **Conventional Commits**: Encourages conventional commit format
- **Imperative Mood**: Suggests using imperative mood
- **Content Validation**: Prevents:
  - WIP/TODO markers in commit messages
  - Inappropriate language
  - Trailing periods
- **Format Checking**: Validates proper commit message structure

## Installation

### Automatic Installation
```bash
# Install hooks using the provided script
make install-hooks

# Or run directly
./.githooks/install.sh
```

### Manual Installation
```bash
# Copy hooks to git hooks directory
cp .githooks/pre-commit .git/hooks/
cp .githooks/pre-push .git/hooks/
cp .githooks/commit-msg .git/hooks/

# Make them executable
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
chmod +x .git/hooks/commit-msg
```

## Testing Hooks

```bash
# Test if hooks are properly installed
make test-hooks

# Test pre-commit hook manually
.git/hooks/pre-commit

# Test commit message hook
echo "test commit message" | .git/hooks/commit-msg /dev/stdin
```

## Bypassing Hooks

Sometimes you may need to bypass hooks (use sparingly):

```bash
# Skip pre-commit hook
git commit --no-verify

# Skip pre-push hook
git push --no-verify
```

## Hook Configuration

### Required Tools
The hooks will automatically check for and suggest installation of:

- `rustfmt` - Code formatting
- `clippy` - Linting
- `cargo-audit` - Security auditing (optional)
- `cargo-outdated` - Dependency checking (optional)

### Environment Variables
You can customize hook behavior with environment variables:

```bash
# Skip certain checks
export SKIP_TESTS=1          # Skip test execution
export SKIP_CLIPPY=1         # Skip clippy checks
export SKIP_FORMAT=1         # Skip format checks
export SKIP_AUDIT=1          # Skip security audit
```

## Troubleshooting

### Hook Not Running
1. Check if hook is executable: `ls -la .git/hooks/`
2. Verify hook location: hooks should be in `.git/hooks/`
3. Check for syntax errors: run hook manually

### Hook Failing
1. Read the error message carefully
2. Fix the reported issues
3. Run the hook manually to test: `.git/hooks/pre-commit`
4. For persistent issues, use `--no-verify` temporarily

### Performance Issues
If hooks are too slow:
1. Consider running only essential checks in pre-commit
2. Move comprehensive checks to pre-push only
3. Use `SKIP_*` environment variables for development

## Customization

### Adding Custom Checks
Edit the hook files to add project-specific checks:

```bash
# Add custom check to pre-commit
vim .githooks/pre-commit

# Add custom validation to commit-msg
vim .githooks/commit-msg
```

### Project-Specific Configuration
Create `.git-hooks-config` in project root:

```bash
# Example configuration
SKIP_TESTS_ON_COMMIT=1
REQUIRE_ISSUE_REFERENCE=1
MAX_COMMIT_LENGTH=50
```

## Best Practices

1. **Keep Hooks Fast**: Pre-commit hooks should complete in <30 seconds
2. **Fail Fast**: Exit early on first error to save time
3. **Clear Messages**: Provide helpful error messages with suggestions
4. **Consistent Environment**: Ensure hooks work across different development environments
5. **Documentation**: Keep this README updated with any changes

## Hook Maintenance

### Updating Hooks
1. Modify hook files in `.githooks/`
2. Test changes thoroughly
3. Run `make install-hooks` to update installed hooks
4. Commit changes to version control

### Sharing Hooks
Hooks are stored in `.githooks/` and committed to the repository, making them available to all team members.

## Security Considerations

- Hooks scan for potential secrets and credentials
- Sensitive patterns are configurable in the hook files
- Consider adding project-specific sensitive patterns
- Regular security audits are performed automatically

## Integration with CI/CD

These hooks complement the GitHub Actions CI/CD pipeline:
- Hooks provide fast local feedback
- CI/CD provides comprehensive testing across platforms
- Both use the same quality standards (clippy, formatting, tests)

For more information about the CI/CD pipeline, see the main project README.