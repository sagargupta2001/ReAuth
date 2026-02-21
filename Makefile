.PHONY: run-before-raising-pr fmt clippy test test-docs coverage ui-lint ui-test

# Colors
GREEN  := $(shell tput -Txterm setaf 2)
RED    := $(shell tput -Txterm setaf 1)
YELLOW := $(shell tput -Txterm setaf 3)
RESET  := $(shell tput -Txterm sgr0)

run-before-raising-pr:
	@echo "$(YELLOW)Starting pre-PR validation...$(RESET)"
	@$(MAKE) fmt || (echo "$(RED)Formatting check failed!$(RESET)" && exit 1)
	@$(MAKE) clippy || (echo "$(RED)Clippy lints failed!$(RESET)" && exit 1)
	@$(MAKE) test || (echo "$(RED)Backend tests failed!$(RESET)" && exit 1)
	@$(MAKE) test-docs || (echo "$(RED)Documentation tests failed!$(RESET)" && exit 1)
	@$(MAKE) coverage || (echo "$(RED)Coverage report generation failed!$(RESET)" && exit 1)
	@$(MAKE) ui-lint || (echo "$(RED)UI Linting failed!$(RESET)" && exit 1)
	@$(MAKE) ui-test || (echo "$(RED)UI Tests failed!$(RESET)" && exit 1)
	@echo "\n$(GREEN)==================================================$(RESET)"
	@echo "$(GREEN)✅ ALL CHECKS PASSED! You are ready to raise a PR.$(RESET)"
	@echo "$(GREEN)==================================================$(RESET)\n"

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
	cd ui && npm run lint

ui-test:
	cd ui && npm run test
