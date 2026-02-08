# 🚀 Quick Fix - New Version with Debugging
## The Problem

The database wasn't being created and events weren't being captured.

## The Solution ✅

I've updated the extension with:
- **Output Channel** for debugging (see exactly what's happening)
- **Better error messages**
- **Simplified database initialization**
- **Document change monitoring** (captures Copilot acceptances)

---

## 📦 Install the Updated Version

### 1. Uninstall Old Version (if installed)
```
VSCode > Extensions > Search "Agent Monitor" > Uninstall
Reload VSCode
```

### 2. Install New Version
```
Extensions > "..." menu > Install from VSIX
Select: vscode-agent-monitor-0.1.0.vsix
Click "Install"
Reload VSCode
```

---

## 🔍 Step-by-Step Test

### 1. Open a Project
```
File > Open Folder > Select any project
```

### 2. Check Logs
```
Cmd+Shift+P (or Ctrl+Shift+P)
Type: "Agent Monitor: Show Logs"
Press Enter
```

You should see:
```
🔍 Agent Monitor: Initializing...
Database path: /path/to/your/project/agent_telemetry.db
📊 Creating database...
✅ Database created successfully
👀 Starting to monitor document changes...
```

### 3. Generate Events
Open a file and **type more than 10 characters at once**.

Watch the logs:
```
📝 Change detected in myfile.js (+25 chars)
✅ Captured (1 total)
```

### 4. Save a File
Press `Cmd+S` (or `Ctrl+S`).

Watch the logs:
```
💾 Document saved: myfile.js
✅ Captured (2 total)
```

### 5. View Dashboard
```
Cmd+Shift+P > "Agent Monitor: Show Dashboard"
```

Should show your captured events!

---

## 🐛 If It Still Doesn't Work

### Check if Cargo is installed:
```bash
cargo --version
```

If not found:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Check the database file:
```bash
# Should exist after activation
ls -lh agent_telemetry.db
```

### Test database creation manually:
```bash
cd /Users/matusdrobuliak/Projects/vscode-agent-monitor
cargo run --example init_db /tmp/test.db
# Should see: ✅ Database initialized
```

---

## 💡 Key Changes

### What's Different:

1. **Visible Logs**: `Cmd+Shift+P` > "Agent Monitor: Show Logs"
   - See exactly what's happening
   - See errors immediately
   - See capture count

2. **Better Capture**: Monitors document changes
   - When you type >10 chars
   - When you save files
   - Includes Copilot acceptances

3. **Simpler Init**: Uses `init_db.rs` example
   - Clearer error messages
   - Shows progress in logs

---

## ✅ Success Checklist

After installing, you should have:

- [ ] Notification: "Agent Monitor: Monitoring active..."
- [ ] Logs show: "Database created successfully"
- [ ] File exists: `agent_telemetry.db` in project folder
- [ ] Typing shows: "Change detected" in logs
- [ ] Dashboard shows: Captured events

---

## 📊 New Command

**Show Logs** - See what's happening:
```
Cmd+Shift+P > "Agent Monitor: Show Logs"
```

This is your best debugging tool!

---

## 🎯 What to Expect

Normal workflow:
1. Open project → Database auto-created
2. Type code → Events captured (see in logs)
3. View dashboard → See captured data
4. Keep coding → More events accumulate

The logs show everything, so you can always see if it's working!
