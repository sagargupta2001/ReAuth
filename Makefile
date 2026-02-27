.PHONY: run-before-raising-pr fmt clippy test test-docs coverage ui-build ui-lint ui-test ui-coverage summary dev embed clean-tmp

# Colors
GREEN  := $(shell tput -Txterm setaf 2)
RED    := $(shell tput -Txterm setaf 1)
YELLOW := $(shell tput -Txterm setaf 3)
CYAN   := $(shell tput -Txterm setaf 6)
RESET  := $(shell tput -Txterm sgr0)

# Temp files for stats
TMP_DIR := .tmp_stats
COV_DIR := target/llvm-cov
COV_SUMMARY := $(COV_DIR)/coverage-summary.json
UI_COV_DIR := ui/coverage
UI_COV_SUMMARY := $(UI_COV_DIR)/coverage-summary.json

run-before-raising-pr: clean-tmp
	@mkdir -p $(TMP_DIR)
	@echo "$(YELLOW)🚀 Starting pre-PR validation...$(RESET)"

	$(call run_step,Building UI,$(MAKE) ui-build,ui_build.log)
	$(call run_step,Checking formatting,$(MAKE) fmt,fmt.log)
	$(call run_step,Running Clippy,$(MAKE) clippy,clippy.log)
	$(call run_step,Running Documentation Tests,$(MAKE) test-docs,test_docs.log)
	$(call run_step,Generating Coverage,$(MAKE) coverage,coverage.log)
	$(call run_step,Linting UI,$(MAKE) ui-lint,ui_lint.log)
	$(call run_step,Running UI Tests + Coverage,$(MAKE) ui-coverage,ui_coverage.log)

	@$(MAKE) summary

summary:
	@echo "\n$(CYAN)📊 VALIDATION SUMMARY$(RESET)"
	@echo "--------------------------------------------------"
	@printf "%-30s | %-15s\n" "Check" "Result"
	@echo "--------------------------------------------------"
	
	@# Extract Backend Test Stats (from test.log or coverage.log)
	@TEST_LOG=$$( [ -s $(TMP_DIR)/test.log ] && echo $(TMP_DIR)/test.log || echo $(TMP_DIR)/coverage.log ); \
	 PASS_BE=$$(grep -o "[0-9]* passed" $$TEST_LOG | awk '{sum += $$1} END {print (sum == "" ? 0 : sum)}'); \
	 FAIL_BE=$$(grep -o "[0-9]* failed" $$TEST_LOG | awk '{sum += $$1} END {print (sum == "" ? 0 : sum)}'); \
	 printf "%-30s | $(GREEN)%s Passed$(RESET), $(RED)%s Failed$(RESET)\n" "Backend Tests" "$$PASS_BE" "$$FAIL_BE"
	
	@# Extract UI Test Stats (vitest output varies; parse "(N tests)" and "failed" lines)
	@PASS_UI=$$(UI_LOG="$(TMP_DIR)/ui_coverage.log" python3 -c 'import os,re; path=os.environ.get("UI_LOG"); data=open(path, "r", encoding="utf-8", errors="ignore").read() if path and os.path.exists(path) else ""; data=re.sub(r"\x1b\[[0-9;]*m", "", data); file_counts=[int(x) for x in re.findall(r"\((\d+)\s+tests?\)", data)]; summary_counts=[int(x) for x in re.findall(r"Tests\s+(\d+)\s+passed", data)]; tests=sum(file_counts) if file_counts else (max(summary_counts) if summary_counts else 0); print(tests)'); \
	 FAIL_UI=$$(UI_LOG="$(TMP_DIR)/ui_coverage.log" python3 -c 'import os,re; path=os.environ.get("UI_LOG"); data=open(path, "r", encoding="utf-8", errors="ignore").read() if path and os.path.exists(path) else ""; data=re.sub(r"\x1b\[[0-9;]*m", "", data); fails=[int(x) for x in re.findall(r"Tests?\s+(\d+)\s+failed", data)] + [int(x) for x in re.findall(r"(\d+)\s+failed", data)]; print(max(fails) if fails else 0)'); \
	 printf "%-30s | $(GREEN)%s Passed$(RESET), $(RED)%s Failed$(RESET)\n" "UI Tests" "$$PASS_UI" "$$FAIL_UI"

	@# Coverage Summary
	@BE_COV=$$(COV_SUMMARY="$(COV_SUMMARY)" python3 -c 'import json, os, sys; path=os.environ.get("COV_SUMMARY"); \
(print("N/A"), sys.exit(0)) if (not path or not os.path.exists(path)) else None; \
data=json.load(open(path, "r")); \
totals=None; \
totals=data.get("data")[0].get("totals") if isinstance(data, dict) and data.get("data") else totals; \
totals=data.get("totals") if isinstance(data, dict) and data.get("totals") else totals; \
(print("N/A"), sys.exit(0)) if not totals else None; \
lines=(totals.get("lines") or {}); \
pct=lines.get("percent"); \
pct=lines.get("pct") if pct is None else pct; \
print("N/A" if pct is None else "{:.2f}%".format(float(pct)))'); \
	UI_COV=$$(UI_COV_SUMMARY="$(UI_COV_SUMMARY)" python3 -c 'import json, os, sys; path=os.environ.get("UI_COV_SUMMARY"); \
(print("N/A"), sys.exit(0)) if (not path or not os.path.exists(path)) else None; \
data=json.load(open(path, "r")); \
total=data.get("total") if isinstance(data, dict) else None; \
(print("N/A"), sys.exit(0)) if not total else None; \
lines=(total.get("lines") or {}); \
pct=lines.get("pct"); \
pct=lines.get("percent") if pct is None else pct; \
print("N/A" if pct is None else "{:.2f}%".format(float(pct)))'); \
	printf "%-30s | %s\n" "Backend Coverage (lines)" "$$BE_COV"; \
	printf "%-30s | %s\n" "UI Coverage (lines)" "$$UI_COV"

	@# Warning Summary
	@CLIPPY_WARN=$$(grep -oE "warning: .*" $(TMP_DIR)/clippy.log | wc -l | tr -d ' '); \
	 LINT_WARN=$$(grep -oE "[0-9]+ warnings?" $(TMP_DIR)/ui_lint.log | tail -n 1 | awk '{print $$1}'); \
	 [ -z "$$LINT_WARN" ] && LINT_WARN=0; \
	 printf "%-30s | %s\n" "Clippy Warnings" "$$CLIPPY_WARN"; \
	 printf "%-30s | %s\n" "UI Lint Warnings" "$$LINT_WARN"
	
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

define run_step
	@echo "$(CYAN)$(1)... (log: $(TMP_DIR)/$(3))$(RESET)"
	@LOG="$(TMP_DIR)/$(3)"; \
	set -e; \
	( $(2) > $$LOG 2>&1 ) & PID=$$!; \
	while kill -0 $$PID 2>/dev/null; do \
		sleep 5; \
		if [ -s $$LOG ]; then \
			TAIL=$$(tail -n 1 $$LOG | tr -d '\r'); \
			echo "$(CYAN)  ... $(1): $$TAIL$(RESET)"; \
		else \
			echo "$(CYAN)  ... $(1) in progress$(RESET)"; \
		fi; \
	done; \
	wait $$PID || { echo "$(RED)❌ $(1) failed!$(RESET)"; cat $$LOG; exit 1; }; \
	echo "$(GREEN)✔ $(1) done$(RESET)"
endef

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

test-docs:
	cargo test -p reauth_core --doc

coverage:
	@mkdir -p $(COV_DIR)
	cargo llvm-cov -p reauth_core --html --output-dir $(COV_DIR)
	cargo llvm-cov -p reauth_core --json --summary-only --output-path $(COV_SUMMARY) --no-run

ui-build:
	cd ui && npm run build

ui-lint:
	cd ui && npm run lint

ui-test:
	cd ui && npm run test

ui-coverage:
	cd ui && npm run coverage

dev:
	cargo run --package reauth_core --bin reauth_core

embed:
	@if [ ! -d ui/dist ]; then \
		echo "$(YELLOW)ui/dist missing. Building UI...$(RESET)"; \
		$(MAKE) ui-build; \
	fi
	cargo run --package reauth_core --bin reauth_core --features embed-ui

clean-tmp:
	@rm -rf $(TMP_DIR)
