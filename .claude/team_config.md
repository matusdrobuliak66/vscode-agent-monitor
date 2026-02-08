# Agent Team Configuration

## Team Structure
4 specialized agents working in parallel:

### Agent 1: Security Specialist
- **Focus**: security_scanner module
- **Tasks**: Write security tests, implement detection logic
- **Skills**: Regex patterns, entropy calculations
- **Memory scope**: project

### Agent 2: Telemetry Specialist  
- **Focus**: telemetry_collector module
- **Tasks**: LSP interception, data extraction
- **Dependencies**: Wait for security_scanner completion
- **Memory scope**: project

### Agent 3: Storage Specialist
- **Focus**: storage_manager module
- **Tasks**: SQLite schema, file hashing, persistence
- **Dependencies**: Wait for telemetry_collector
- **Memory scope**: project

### Agent 4: Integration Validator
- **Focus**: Cross-module testing and validation
- **Tasks**: Run full test suite, check for regressions
- **Challenge**: Question other agents' implementations
- **Memory scope**: project

## Coordination Rules
- Agents must update PROGRESS.md after completing each task
- Move completed task files from current_tasks/ to completed_tasks/
- Agents should challenge each other's findings
- Security agent validates all code for vulnerability risks
