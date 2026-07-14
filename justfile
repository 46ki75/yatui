set shell := ["bash", "-cu"]

default: ci

fmt:
    cargo fmt --all
    pnpm run lint:fix

fmt-check:
    cargo fmt --all -- --check
    pnpm run lint

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

ci: fmt-check lint test doc

test-cov:
    cargo llvm-cov --no-report --workspace --all-features

coverage: test-cov
    cargo llvm-cov report --show-missing-lines --color=always

coverage-html: test-cov
    cargo llvm-cov report --html --open

coverage-ci: test-cov
    cargo llvm-cov report --lcov --output-path lcov.info
