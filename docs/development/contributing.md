# Contributing Guidelines

This document outlines the process for contributing to the Sonoma Search engine project.

## Code of Conduct

We are committed to providing a friendly, safe, and welcoming environment for all contributors. Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/rainsfal1/sonoma-search.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Set up your development environment following the instructions in `docs/development/environment.md`

## Development Process

### 1. Coding Standards

#### Rust Code
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for code formatting
- Ensure `clippy` reports no warnings
- Write documentation for public APIs
- Include unit tests for new functionality

```rust
/// Example of good documentation and code style
/// 
/// # Arguments
/// * `query` - The search query string
/// * `limit` - Maximum number of results to return
/// 
/// # Returns
/// A vector of search results
pub fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, Error> {
    // Implementation
}
```

#### Commit Messages
Follow the conventional commits specification:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation changes
- style: Code style changes
- refactor: Code refactoring
- perf: Performance improvements
- test: Adding or modifying tests
- chore: Maintenance tasks

Example:
```
feat(searcher): add fuzzy search capability

Implement Levenshtein distance algorithm for fuzzy matching.
Includes unit tests and documentation.

Closes #123
```

### 2. Testing Requirements

- Write unit tests for new code
- Update existing tests when modifying functionality
- Include integration tests for API changes
- Add benchmarks for performance-critical code

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_basic() {
        // Test implementation
    }

    #[test]
    fn test_search_edge_cases() {
        // Test implementation
    }

    #[test]
    fn test_search_error_handling() {
        // Test implementation
    }
}
```

### 3. Documentation

- Update relevant documentation for new features
- Include inline documentation for public APIs
- Add examples for complex functionality
- Update README.md if necessary

### 4. Pull Request Process

1. Update your branch with the latest changes from main:
```bash
git fetch origin
git rebase origin/main
```

2. Run all tests and checks:
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

3. Create a pull request with:
   - Clear title following commit message format
   - Detailed description of changes
   - Reference to related issues
   - Screenshots/videos for UI changes
   - List of testing steps

4. Address review feedback:
   - Respond to all comments
   - Make requested changes
   - Push updates to your branch

5. Maintain your PR:
   - Resolve merge conflicts
   - Update outdated code
   - Respond to CI failures

## Review Process

### Reviewer Guidelines

1. Code Quality
   - Check for clean and maintainable code
   - Verify proper error handling
   - Ensure logging is appropriate
   - Look for potential performance issues

2. Testing
   - Verify test coverage
   - Check test quality and edge cases
   - Ensure tests are meaningful

3. Documentation
   - Check for clear and complete documentation
   - Verify examples are accurate
   - Ensure API documentation is complete

### Author Responsibilities

1. Be responsive to feedback
2. Test changes thoroughly
3. Keep the PR focused and small
4. Update the PR based on reviews
5. Maintain a professional attitude

## Release Process

### Version Numbers

Follow semantic versioning (MAJOR.MINOR.PATCH):
- MAJOR: Breaking changes
- MINOR: New features
- PATCH: Bug fixes

### Release Checklist

1. Update version numbers
2. Update CHANGELOG.md
3. Create release notes
4. Tag the release
5. Deploy to staging
6. Verify deployment
7. Deploy to production

## Reporting Issues

### Bug Reports

Include:
1. Description of the bug
2. Steps to reproduce
3. Expected behavior
4. Actual behavior
5. Environment details
6. Relevant logs
7. Screenshots if applicable

Example:
```markdown
**Bug Description**
Search results are not properly sorted by relevance.

**Steps to Reproduce**
1. Navigate to search page
2. Enter query "test search"
3. Observe results order

**Expected Behavior**
Results should be sorted by relevance score.

**Actual Behavior**
Results appear in random order.

**Environment**
- OS: Ubuntu 22.04
- Browser: Chrome 120
- Backend Version: 1.2.3
```

### Feature Requests

Include:
1. Clear description of the feature
2. Use cases
3. Expected benefits
4. Potential implementation approach
5. Mockups/diagrams if applicable

## Community

### Communication Channels

- GitHub Issues: Bug reports and feature requests
- Pull Requests: Code review discussions
- Project Board: Task tracking
- Documentation: Technical references

### Getting Help

1. Check existing documentation
2. Search closed issues
3. Ask in appropriate channel
4. Provide context and examples

## License

By contributing, you agree that your contributions will be licensed under the project's license. 