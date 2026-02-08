// Event Browser - Interactive UI for filtering and viewing events

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { execSync } from 'child_process';

export class EventBrowserPanel {
    private static currentPanel: vscode.WebviewPanel | undefined;
    private dbPath: string;
    private extensionRoot: string;

    private constructor(dbPath: string, extensionRoot: string, panel: vscode.WebviewPanel) {
        this.dbPath = dbPath;
        this.extensionRoot = extensionRoot;

        panel.webview.html = this.getLoadingHtml();

        // Set up message handling
        panel.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.command) {
                    case 'loadEvents':
                        await this.loadEvents(panel, message.filters);
                        break;
                    case 'viewContext':
                        await this.viewContext(panel, message.telemetryId);
                        break;
                }
            }
        );

        // Load initial events
        this.loadEvents(panel, {});
    }

    public static show(dbPath: string, extensionRoot: string) {
        const column = vscode.ViewColumn.One;

        if (EventBrowserPanel.currentPanel) {
            EventBrowserPanel.currentPanel.reveal(column);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'agentMonitorEventBrowser',
            'Agent Monitor: Event Browser',
            column,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        EventBrowserPanel.currentPanel = panel;

        panel.onDidDispose(() => {
            EventBrowserPanel.currentPanel = undefined;
        });

        new EventBrowserPanel(dbPath, extensionRoot, panel);
    }

    private getLoadingHtml(): string {
        return `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Event Browser</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-editor-foreground);
            background: var(--vscode-editor-background);
            padding: 20px;
        }
        .loading {
            text-align: center;
            padding: 40px;
        }
    </style>
</head>
<body>
    <div class="loading">
        <h2>Loading events...</h2>
    </div>
</body>
</html>`;
    }

    private async loadEvents(panel: vscode.WebviewPanel, filters: any) {
        try {
            // Build command with filters
            let cmd = `cargo run --example query_events "${this.dbPath}"`;

            if (filters.method) {
                cmd += ` --method "${filters.method}"`;
            }
            if (filters.limit) {
                cmd += ` --limit ${filters.limit}`;
            }

            const output = execSync(cmd, {
                cwd: this.extensionRoot,
                encoding: 'utf-8',
                timeout: 10000
            });

            const events = JSON.parse(output);
            panel.webview.html = this.generateEventBrowserHtml(events, filters);
        } catch (error: any) {
            vscode.window.showErrorMessage(`Failed to load events: ${error.message}`);
        }
    }

    private async viewContext(panel: vscode.WebviewPanel, telemetryId: number) {
        try {
            const output = execSync(
                `cargo run --example get_finding_context "${this.dbPath}" ${telemetryId}`,
                {
                    cwd: this.extensionRoot,
                    encoding: 'utf-8',
                    timeout: 5000
                }
            );

            const data = JSON.parse(output);
            this.showContextModal(panel, data);
        } catch (error: any) {
            vscode.window.showErrorMessage(`Failed to load context: ${error.message}`);
        }
    }

    private showContextModal(panel: vscode.WebviewPanel, data: any) {
        // Send data to webview to show in modal
        panel.webview.postMessage({
            command: 'showContext',
            data: data
        });
    }

    private generateEventBrowserHtml(events: any[], filters: any): string {
        // Get unique methods for filter dropdown
        const methods = [...new Set(events.map(e => e.method))];

        return `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Event Browser</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-editor-foreground);
            background: var(--vscode-editor-background);
            padding: 20px;
        }

        .header {
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 1px solid var(--vscode-panel-border);
        }

        h1 {
            font-size: 24px;
            margin-bottom: 10px;
        }

        .filters {
            display: flex;
            gap: 15px;
            align-items: center;
            margin-top: 15px;
        }

        select, button {
            background: var(--vscode-input-background);
            color: var(--vscode-input-foreground);
            border: 1px solid var(--vscode-input-border);
            padding: 6px 12px;
            border-radius: 3px;
            font-size: 13px;
        }

        button {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            cursor: pointer;
        }

        button:hover {
            background: var(--vscode-button-hoverBackground);
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }

        thead {
            background: var(--vscode-editor-inactiveSelectionBackground);
        }

        th {
            text-align: left;
            padding: 10px;
            font-weight: 600;
            border-bottom: 1px solid var(--vscode-panel-border);
        }

        td {
            padding: 10px;
            border-bottom: 1px solid var(--vscode-panel-border);
        }

        tr:hover {
            background: var(--vscode-list-hoverBackground);
        }

        .method-badge {
            display: inline-block;
            padding: 3px 8px;
            background: var(--vscode-badge-background);
            color: var(--vscode-badge-foreground);
            border-radius: 3px;
            font-size: 11px;
            font-family: monospace;
        }

        .finding-count {
            display: inline-block;
            padding: 2px 6px;
            background: #ff4444;
            color: white;
            border-radius: 10px;
            font-size: 11px;
            font-weight: bold;
        }

        .finding-count.zero {
            background: #4CAF50;
        }

        .view-btn {
            padding: 4px 10px;
            font-size: 12px;
        }

        .modal {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.7);
            z-index: 1000;
        }

        .modal-content {
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-panel-border);
            border-radius: 8px;
            width: 80%;
            max-width: 900px;
            max-height: 80%;
            margin: 50px auto;
            overflow: auto;
            padding: 20px;
        }

        .modal-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 1px solid var(--vscode-panel-border);
        }

        .close-btn {
            cursor: pointer;
            font-size: 24px;
            color: var(--vscode-editor-foreground);
        }

        .context-section {
            margin-bottom: 20px;
        }

        .context-section h3 {
            margin-bottom: 10px;
            color: var(--vscode-focusBorder);
        }

        pre {
            background: var(--vscode-textCodeBlock-background);
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
            font-size: 13px;
            line-height: 1.5;
        }

        .finding-badge {
            display: inline-block;
            padding: 4px 10px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: bold;
            margin-right: 10px;
        }

        .finding-badge.High {
            background: #ff4444;
            color: white;
        }

        .finding-badge.Medium {
            background: #ff8c00;
            color: white;
        }

        .finding-badge.Low {
            background: #ffd700;
            color: black;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>📊 Event Browser</h1>
        <div class="filters">
            <label>Filter by method:</label>
            <select id="methodFilter">
                <option value="">All Methods</option>
                ${methods.map(m => `<option value="${m}" ${filters.method === m ? 'selected' : ''}>${m}</option>`).join('')}
            </select>
            <button onclick="applyFilters()">Apply Filters</button>
            <button onclick="clearFilters()">Clear</button>
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>ID</th>
                <th>Timestamp</th>
                <th>Method</th>
                <th>Findings</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>
            ${events.map(e => `
                <tr>
                    <td>${e.id}</td>
                    <td>${new Date(e.timestamp * 1000).toLocaleString()}</td>
                    <td><span class="method-badge">${this.htmlEscape(e.method)}</span></td>
                    <td><span class="finding-count ${e.findingCount === 0 ? 'zero' : ''}">${e.findingCount}</span></td>
                    <td><button class="view-btn" onclick="viewContext(${e.id})">View Context</button></td>
                </tr>
            `).join('')}
        </tbody>
    </table>

    <div id="contextModal" class="modal">
        <div class="modal-content">
            <div class="modal-header">
                <h2>Event Context</h2>
                <span class="close-btn" onclick="closeModal()">&times;</span>
            </div>
            <div id="contextBody"></div>
        </div>
    </div>

    <script>
        const vscode = acquireVsCodeApi();

        function applyFilters() {
            const method = document.getElementById('methodFilter').value;
            vscode.postMessage({
                command: 'loadEvents',
                filters: { method: method || undefined, limit: 100 }
            });
        }

        function clearFilters() {
            vscode.postMessage({
                command: 'loadEvents',
                filters: {}
            });
        }

        function viewContext(telemetryId) {
            vscode.postMessage({
                command: 'viewContext',
                telemetryId: telemetryId
            });
        }

        function closeModal() {
            document.getElementById('contextModal').style.display = 'none';
        }

        // Handle messages from extension
        window.addEventListener('message', event => {
            const message = event.data;
            if (message.command === 'showContext') {
                showContextModal(message.data);
            }
        });

        function showContextModal(data) {
            const event = data.event;
            const findings = data.findings;

            let html = \`
                <div class="context-section">
                    <h3>Event Details</h3>
                    <p><strong>ID:</strong> \${event.id}</p>
                    <p><strong>Method:</strong> <span class="method-badge">\${event.method}</span></p>
                    <p><strong>Timestamp:</strong> \${new Date(event.timestamp * 1000).toLocaleString()}</p>
                </div>

                <div class="context-section">
                    <h3>Security Findings (\${findings.length})</h3>
                    \${findings.map(f => \`
                        <div style="margin-bottom: 10px;">
                            <span class="finding-badge \${f.severity}">\${f.severity}</span>
                            <strong>\${f.patternType}</strong> at line \${f.lineNum}
                        </div>
                    \`).join('')}
                </div>

                <div class="context-section">
                    <h3>Request Content</h3>
                    <pre>\${event.request}</pre>
                </div>

                \${event.response ? \`
                    <div class="context-section">
                        <h3>Response Content</h3>
                        <pre>\${event.response}</pre>
                    </div>
                \` : ''}
            \`;

            document.getElementById('contextBody').innerHTML = html;
            document.getElementById('contextModal').style.display = 'block';
        }
    </script>
</body>
</html>`;
    }

    private htmlEscape(text: string): string {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#x27;');
    }
}
