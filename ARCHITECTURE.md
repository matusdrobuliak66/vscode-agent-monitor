# Architecture

## Module Dependencies
security_scanner (no dependencies)
↓
telemetry_collector (uses security_scanner)
↓
storage_manager (uses telemetry_collector)
↓
visualization_panel (uses storage_manager)

## Data Flow
1. VSCode agent makes LLS request
2. telemetry_collector intercepts via LSP hooks
3. security_scanner checks context for secrets
4. storage_manager persists to SQLite + files
5. visualization_panel queries and displays

## Security Design
- Context files stored with content-hash filenames
- Security scanner runs before storage
- Flagged content gets severity levels
- Export feature for manual auditing