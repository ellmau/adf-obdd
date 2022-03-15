# What does this PR do?

* Describe what the PR does
* Reference issue/pr/discussion numbers and use the appropriate github-keywords

# Checklist before creating a non-draft PR

- [ ] All tests are passing
- [ ] Clippy has no complains
- [ ] Code is `rustfmt` formatted
- [ ] Applicable labels are chosen (Note: it is not necessary to replicate the labels from the related issues)
- [ ] There are no other open [Pull Requests](https://github.com/ellmau/adf-obdd/pulls) for the same update/change.
  - [ ] If there is a good reason to have another PR for the same update/change, it is well justified.

# Checklist on Guidelines and Conventions

- [ ] Commit messages follow our guidelines
- [ ] Code is self-reviewed
- [ ] Naming conventions are met
- [ ] New features are tested
  - [ ] `quickcheck` has been considered
  - [ ] All variants are considered and checked
- Clippy Compiler-exceptions
  - [ ] Used in a sparse manner
  - [ ] If used, a separate comment describes and justifies its use
- [ ] `rustdoc` comments are self-reviewed and descriptive
- Error handling
  - [ ] Use of `panic!(...)` applications is justified on non-recoverable situations
  - [ ] `expect(...)` is used over `unwrap()` (except obvious test-cases)
- [ ] No unsafe code (exceptions need to be discussed specifically)
