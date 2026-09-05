# Everything. If this passes, the code is ready to ship.
default: fmt-check clippy test-all doc

# All the tests.
test-all: test test-bare test-doc

# `typesift_test_suite` depends on `typesift` with every feature enabled, so
# this single command covers all of them.
[doc("The main suite, every feature enabled.")]
test:
    cargo test --workspace --all-features

# Needs `-p`: building the whole workspace pulls in the test suite, which turns
# every feature back on through feature unification.
[doc("`typesift` alone, without the optional dependencies.")]
test-bare:
    cargo test -p typesift --no-default-features

# The test suite's `compile_fail` module is `#[cfg(doctest)]`, so the
# compile-fail cases run here and nowhere else.
[doc("Doc tests, including the compile-fail cases.")]
test-doc:
    cargo test --doc -p typesift --all-features
    cargo test --doc -p typesift_test_suite

# Format the code.
fmt:
    cargo fmt --all

# This is the one `default` runs, so that a full check never rewrites files.
[doc("Fail if the code is not formatted.")]
fmt-check:
    cargo fmt --all --check

# Lint everything, warnings are errors.
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo clippy -p typesift --no-default-features --all-targets -- -D warnings

# The docs build, with no broken intra-doc links.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
