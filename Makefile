.PHONY: run-before-raising-pr fmt clippy test test-docs coverage ui-lint ui-test

run-before-raising-pr: fmt clippy test test-docs coverage ui-lint ui-test

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

test-docs:
	cargo test -p reauth_core --doc

coverage:
	cargo llvm-cov -p reauth_core --html

ui-lint:
	cd ui && npm run lint -- --fix || true

ui-test:
	cd ui && npm run test
