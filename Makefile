.PHONY: run-before-raising-pr fmt clippy test test-docs coverage

run-before-raising-pr: fmt clippy test test-docs coverage

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

test-docs:
	cargo test -p reauth_core --doc

coverage:
	cargo llvm-cov -p reauth_core
