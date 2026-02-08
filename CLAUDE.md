# VSCode Agent Monitor Project

## Project Goal
Build a VSCode extension in Rust that monitors agent-to-model interactions, tracking calls, capturing context, and scanning for security vulnerabilities.

## Architecture
Five independent modules:
- security_scanner: Detects secrets/passwords in captured context
- telemetry_collector: Intercepts LSP calls from agents to models
- storage_manager: Persists interaction logs to SQLite
- visualization_panel: WebView UI showing usage stats
- data_exporter: Exports data for analysis

## Current Status
See PROGRESS.md for latest status.

## Task System
- current_tasks/: Files representing in-progress work
- completed_tasks/: Archived completed task files
- Each task file contains: goal, acceptance criteria, dependencies

## Development Workflow
1. Check tests: `cargo test`
2. Pick next task from current_tasks/
3. Implement feature
4. Verify tests pass
5. Update PROGRESS.md
6. Move task file to completed_tasks/

## Test Philosophy
- Test-first development
- Security tests are highest priority
- Tests must have clear, grep-friendly output
- Run `cargo test --lib` for fast iteration
