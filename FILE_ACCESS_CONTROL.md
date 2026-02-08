# 🔒 File Access Control for AI Agents - Design Document

## Overview

Control which files AI agents can read or write to prevent sensitive data leakage or unauthorized modifications.

---

## 🎯 Goals

1. **Prevent AI agents from reading sensitive files** (secrets, credentials, private keys)
2. **Prevent AI agents from writing to critical files** (build configs, deployment scripts)
3. **Provide easy configuration** via `.agentignore` or VSCode settings
4. **Audit access attempts** - log when agents try to access restricted files
5. **Flexible rules** - glob patterns, regex, explicit paths

---

## 💡 Implementation Approaches

### Approach 1: LSP Request Interception (Recommended)

**How it works:**
- Intercept LSP requests before they reach the AI agent
- Check if requested file is in restricted list
- Block request or return sanitized/empty response
- Log the blocked attempt

**Pros:**
- ✅ Works at protocol level (all AI tools)
- ✅ Agent never sees restricted content
- ✅ Can provide fake/sanitized responses
- ✅ Complete control

**Cons:**
- ❌ Complex to implement (need LSP middleware)
- ❌ May affect performance
- ❌ Need to handle all LSP methods

**Implementation:**
```typescript
class LSPAccessControl {
    private blockedPatterns: string[] = [];

    intercept(request: LSPRequest): LSPRequest | null {
        if (this.isBlocked(request.params.textDocument?.uri)) {
            this.logBlockedAccess(request);
            return null; // Block request
        }
        return request;
    }
}
```

---

### Approach 2: File Watcher + Warning System

**How it works:**
- Monitor file access patterns
- Detect when AI agent requests sensitive files
- Show warning notification to user
- Don't actually block, but log and warn

**Pros:**
- ✅ Easy to implement
- ✅ Non-intrusive (user still in control)
- ✅ Good for auditing
- ✅ Works with any AI tool

**Cons:**
- ❌ Doesn't prevent access (only warns)
- ❌ Reactive, not proactive
- ❌ User must manually intervene

**Implementation:**
```typescript
class FileAccessMonitor {
    checkAccess(filePath: string, agent: string) {
        if (this.isRestricted(filePath)) {
            vscode.window.showWarningMessage(
                `⚠️ AI Agent attempting to access restricted file: ${filePath}`,
                'Allow Once', 'Block', 'Configure Rules'
            );
        }
    }
}
```

---

### Approach 3: Configuration-Based Content Filtering

**How it works:**
- Define rules in `.agentignore` file
- When agent requests file, check rules
- Return sanitized version (remove secrets, redact sections)
- Track what was redacted

**Pros:**
- ✅ Agent can still work with file structure
- ✅ Flexible redaction rules
- ✅ User-friendly configuration
- ✅ Good balance of security and usability

**Cons:**
- ❌ Complex redaction logic
- ❌ May miss sensitive patterns
- ❌ Agent sees partial content

**Example `.agentignore`:**
```gitignore
# Completely block these files
.env
.env.*
secrets.json
**/credentials/**

# Redact sensitive sections
# [pattern] [action]
*.yaml: redact-env-vars
config.json: redact-api-keys
```

---

### Approach 4: Virtual File System (Advanced)

**How it works:**
- Create virtual FS layer between AI agent and real FS
- Control all read/write operations
- Can provide fake data for restricted files
- Full access control

**Pros:**
- ✅ Complete control
- ✅ Can provide fake data (honeypot)
- ✅ Works for all file operations
- ✅ Most secure

**Cons:**
- ❌ Very complex to implement
- ❌ May break some tools
- ❌ Performance overhead
- ❌ Requires deep VS Code integration

---

## 🎨 Recommended Solution: Hybrid Approach

Combine multiple approaches for best results:

### Phase 1: Monitoring & Logging (Easy - Start Here)
```typescript
// Implement file access monitoring
- Track all files accessed during AI interactions
- Log to database with agent info
- Show in Event Browser
- Weekly summary of accessed files
```

**Implementation Time:** 2-3 hours
**Complexity:** Low
**Value:** High (visibility into AI behavior)

### Phase 2: Rule-Based Warnings (Medium)
```typescript
// Add .agentignore configuration
- Parse .agentignore file (git ignore syntax)
- Check file access against rules
- Show warnings when restricted files accessed
- Allow user to approve/deny
```

**Implementation Time:** 1 day
**Complexity:** Medium
**Value:** High (prevents accidental leaks)

### Phase 3: Content Filtering (Advanced)
```typescript
// Implement smart redaction
- Detect sensitive patterns in files
- Redact before sending to AI
- Track what was redacted
- Configurable redaction rules
```

**Implementation Time:** 2-3 days
**Complexity:** High
**Value:** Medium (most users prefer blocking)

---

## 📝 Proposed Configuration Format

### `.agentignore` File

```gitignore
# Complete block - agent cannot read at all
.env
.env.*
secrets.json
**/credentials/**
*.pem
*.key
id_rsa*

# Redact specific patterns
config.yaml: [redact-env-vars, redact-api-keys]
docker-compose.yml: [redact-passwords]

# Write-protected (can read, cannot write)
!write: package.json
!write: Cargo.toml
!write: tsconfig.json
!write: **/.github/workflows/**

# Temporary allow (for specific session)
+allow-once: deployment/secrets.yaml
```

### VSCode Settings

```json
{
    "agentMonitor.accessControl": {
        "enabled": true,
        "mode": "warn", // "warn" | "block" | "log-only"
        "blockPatterns": [
            ".env*",
            "secrets/**",
            "*.pem"
        ],
        "writeProtectedPatterns": [
            "package.json",
            "**/workflows/**"
        ],
        "allowTemporary": true,
        "showNotifications": true,
        "logAllAccess": true
    }
}
```

---

## 🔧 Implementation Plan

### Step 1: Add Access Logging (This Week)
```typescript
// In lspCapture.ts
interface FileAccessLog {
    timestamp: number;
    filePath: string;
    operation: 'read' | 'write';
    agent: string;
    allowed: boolean;
    reason?: string;
}

// Store in database
// Show in Event Browser
```

### Step 2: Add .agentignore Support (Next Week)
```typescript
// New module: accessControl.ts
class AccessControlManager {
    private rules: AccessRule[];

    loadRules() {
        // Parse .agentignore
        // Parse VSCode settings
        // Combine into rules
    }

    checkAccess(filePath: string, operation: 'read' | 'write'): AccessDecision {
        // Check rules
        // Return decision
    }
}
```

### Step 3: Integrate with Extension
```typescript
// In extension.ts
const accessControl = new AccessControlManager();

// Monitor document opens
vscode.workspace.onDidOpenTextDocument(doc => {
    const decision = accessControl.checkAccess(doc.uri.fsPath, 'read');
    if (decision.blocked) {
        // Show warning
        // Log attempt
    }
});
```

---

## 🎯 Quick Win: Start with Monitoring

**Implement This First (30 minutes):**

1. Track file paths in telemetry
2. Show in Event Browser
3. Export file access report

This gives immediate visibility without blocking anything!

---

## 🤔 Discussion Points

1. **How strict should defaults be?**
   - Option A: Permissive (log only, no blocking)
   - Option B: Secure (block common sensitive files by default)
   - Option C: User choice during setup

2. **Should we provide fake data for blocked files?**
   - Pro: Agent continues working
   - Con: May confuse AI, privacy concerns

3. **Temporary allow workflow?**
   - Allow once per session
   - Allow for N minutes
   - Require password/confirmation

4. **Integration with existing tools?**
   - Read `.gitignore` as baseline?
   - Sync with `.env.example` patterns?

---

## 📊 Next Steps

**Let's decide together:**

1. **Which approach do you prefer?**
   - Monitoring + Warnings (easy, safe)
   - Full blocking (secure, may disrupt workflow)
   - Content filtering (complex, flexible)

2. **What should be blocked by default?**
   - `.env*` files?
   - SSH keys (`*.pem`, `id_rsa`)?
   - Secrets folders?

3. **How should warnings work?**
   - Popup notification?
   - Status bar indicator?
   - Logs only?

4. **Implementation priority?**
   - Phase 1 only (monitoring)?
   - Phase 1 + 2 (+ warnings)?
   - All phases (complete solution)?

---

## 🚀 Sample Implementation (Phase 1)

Want me to implement Phase 1 (monitoring) right now? It would:
- Track all files accessed during captures
- Store file paths in database
- Show in Event Browser with filters
- Export file access report

Takes ~30 minutes. Should I proceed?
