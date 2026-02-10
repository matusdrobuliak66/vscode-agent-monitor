.PHONY: help install build build-ts build-rust watch test test-rust test-rust-lib test-ts package publish dev run clean clean-data init-db

DB ?= agent_monitor.db

help: ## Show this help
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

install: ## Install Node dependencies (triggers electron-rebuild)
	npm install

build: build-ts build-rust ## Build everything (TypeScript + Rust)

build-ts: ## Compile TypeScript
	npm run compile

build-rust: ## Build Rust library (release)
	cargo build --release

watch: ## Watch TypeScript for continuous recompilation
	npm run watch

test: test-rust test-ts ## Run all tests

test-rust: ## Run all Rust tests
	cargo test

test-rust-lib: ## Run Rust lib tests only (fast iteration)
	cargo test --lib

test-ts: ## Lint TypeScript
	npm run lint

package: build ## Build VSIX package
	npx @vscode/vsce package --allow-missing-repository --baseContentUrl . --baseImagesUrl .

publish: ## Publish to VS Code Marketplace
	npx @vscode/vsce publish

dev: install build ## Full dev setup, then start watch mode
	npm run watch

run: ## Launch Extension Development Host
	code --extensionDevelopmentPath=.

clean: ## Remove all build artifacts
	rm -rf out/ target/ *.vsix

clean-data: ## Remove telemetry databases
	rm -f *.db *.db-shm *.db-wal

init-db: ## Initialize a fresh database (usage: make init-db DB=/path/to/db)
	cargo run --example init_db -- $(DB)
