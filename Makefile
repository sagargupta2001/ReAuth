.PHONY: run-before-raising-pr fmt clippy test test-docs coverage ui-build ui-lint ui-test clean-tmp

# Colors
GREEN  := $(shell tput -Txterm setaf 2)
RED    := $(shell tput -Txterm setaf 1)
YELLOW := $(shell tput -Txterm setaf 3)
CYAN   := $(shell tput -Txterm setaf 6)
RESET  := $(shell tput -Txterm sgr0)

# Temp files for stats
TMP_DIR := .tmp_stats

run-before-raising-pr: clean-tmp
	@mkdir -p $(TMP_DIR)
	@echo "$(YELLOW)🚀 Starting pre-PR validation...$(RESET)"
	
	@echo "$(CYAN)Building UI...$(RESET)"
	@$(MAKE) ui-build > $(TMP_DIR)/ui_build.log 2>&1 || (echo "$(RED)❌ UI Build failed!$(RESET)" && cat $(TMP_DIR)/ui_build.log && exit 1)
	
	@echo "$(CYAN)Checking formatting...$(RESET)"
	@$(MAKE) fmt > $(TMP_DIR)/fmt.log 2>&1 || (echo "$(RED)❌ Formatting check failed!$(RESET)" && cat $(TMP_DIR)/fmt.log && exit 1)
	
	@echo "$(CYAN)Running Clippy...$(RESET)"
	@$(MAKE) clippy > $(TMP_DIR)/clippy.log 2>&1 || (echo "$(RED)❌ Clippy lints failed!$(RESET)" && cat $(TMP_DIR)/clippy.log && exit 1)
	
	@echo "$(CYAN)Running Backend Tests...$(RESET)"
	@$(MAKE) test > $(TMP_DIR)/test.log 2>&1 || (echo "$(RED)❌ Backend tests failed!$(RESET)" && cat $(TMP_DIR)/test.log && exit 1)
	
	@echo "$(CYAN)Running Documentation Tests...$(RESET)"
	@$(MAKE) test-docs > $(TMP_DIR)/test_docs.log 2>&1 || (echo "$(RED)❌ Documentation tests failed!$(RESET)" && cat $(TMP_DIR)/test_docs.log && exit 1)
	
	@echo "$(CYAN)Generating Coverage...$(RESET)"
	@$(MAKE) coverage > $(TMP_DIR)/coverage.log 2>&1 || (echo "$(RED)❌ Coverage report failed!$(RESET)" && cat $(TMP_DIR)/coverage.log && exit 1)
	
	@echo "$(CYAN)Linting UI...$(RESET)"
	@$(MAKE) ui-lint > $(TMP_DIR)/ui_lint.log 2>&1 || (echo "$(RED)❌ UI Linting failed!$(RESET)" && cat $(TMP_DIR)/ui_lint.log && exit 1)
	
	@echo "$(CYAN)Running UI Tests...$(RESET)"
	@$(MAKE) ui-test > $(TMP_DIR)/ui_test.log 2>&1 || (echo "$(RED)❌ UI Tests failed!$(RESET)" && cat $(TMP_DIR)/ui_test.log && exit 1)

	@$(MAKE) summary

summary:
	@echo "\n$(CYAN)📊 VALIDATION SUMMARY$(RESET)"
	@echo "--------------------------------------------------"
	@printf "%-30s | %-15s\n" "Check" "Result"
	@echo "--------------------------------------------------"
	
	@# Extract Backend Test Stats
	@PASS_BE=$$(grep -o "[0-9]* passed" $(TMP_DIR)/test.log | awk '{sum += $$1} END {print (sum == "" ? 0 : sum)}'); \
	 FAIL_BE=$$(grep -o "[0-9]* failed" $(TMP_DIR)/test.log | awk '{sum += $$1} END {print (sum == "" ? 0 : sum)}'); \
	 printf "%-30s | $(GREEN)%s Passed$(RESET), $(RED)%s Failed$(RESET)\n" "Backend Tests" "$$PASS_BE" "$$FAIL_BE"
	
	@# Extract UI Test Stats
	@PASS_UI=$$(grep -o "[0-9]* passed" $(TMP_DIR)/ui_test.log | head -n 1 | awk '{print $$1}'); \
	 FAIL_UI=$$(grep -o "[0-9]* failed" $(TMP_DIR)/ui_test.log | head -n 1 | awk '{print $$1}'); \
	 [ -z "$$PASS_UI" ] && PASS_UI=0; [ -z "$$FAIL_UI" ] && FAIL_UI=0; \
	 printf "%-30s | $(GREEN)%s Passed$(RESET), $(RED)%s Failed$(RESET)\n" "UI Tests" "$$PASS_UI" "$$FAIL_UI"
	
	@# Lint/Build Status
	@printf "%-30s | $(GREEN)PASS$(RESET)\n" "Rust Formatting (fmt)"
	@printf "%-30s | $(GREEN)PASS$(RESET)\n" "Rust Linting (clippy)"
	@printf "%-30s | $(GREEN)PASS$(RESET)\n" "UI Build"
	@printf "%-30s | $(GREEN)PASS$(RESET)\n" "UI Linting"
	
	@echo "--------------------------------------------------"
	@echo "\n$(GREEN)==================================================$(RESET)"
	@echo "$(GREEN)✅ ALL CHECKS PASSED! You are ready to raise a PR.$(RESET)"
	@echo "$(GREEN)==================================================$(RESET)\n"
	@$(MAKE) clean-tmp

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

ui-build:
	cd ui && npm run build

ui-lint:
	cd ui && npm run lint

ui-test:
	cd ui && npm run test

clean-tmp:
	@rm -rf $(TMP_DIR)
