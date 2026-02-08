import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { execSync } from 'child_process';
import { LSPCaptureManager } from './lspCapture';
import { EventBrowserPanel } from './eventBrowser';

let currentPanel: vscode.WebviewPanel | undefined = undefined;
let captureManager: LSPCaptureManager | undefined = undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('VSCode Agent Monitor is now active!');

    // Initialize LSP capture if workspace is open
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (workspaceFolder) {
        const extensionRoot = context.extensionPath;
        captureManager = new LSPCaptureManager(workspaceFolder.uri.fsPath, extensionRoot);
        captureManager.startCapturing();
        vscode.window.showInformationMessage('Agent Monitor: Now monitoring LSP interactions');
    }

    // Register command to show dashboard
    let showDashboard = vscode.commands.registerCommand('vscode-agent-monitor.showDashboard', () => {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        // Look for the telemetry database
        const dbPath = path.join(workspaceFolder.uri.fsPath, 'agent_telemetry.db');

        if (!fs.existsSync(dbPath)) {
            vscode.window.showWarningMessage('No telemetry data yet. Start coding to capture LSP interactions!');
            return;
        }

        showDashboardPanel(dbPath, context);
    });

    // Register command to show logs
    let showLogs = vscode.commands.registerCommand('vscode-agent-monitor.showLogs', () => {
        if (captureManager) {
            captureManager.showLogs();
        } else {
            vscode.window.showWarningMessage('Agent Monitor: Not active (no workspace open)');
        }
    });

    // Register command to show event browser
    let showEventBrowser = vscode.commands.registerCommand('vscode-agent-monitor.showEventBrowser', () => {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        const dbPath = path.join(workspaceFolder.uri.fsPath, 'agent_telemetry.db');

        if (!fs.existsSync(dbPath)) {
            vscode.window.showWarningMessage('No telemetry data yet. Start coding to capture events!');
            return;
        }

        EventBrowserPanel.show(dbPath, context.extensionPath);
    });

    // Register command to clear data
    let clearData = vscode.commands.registerCommand('vscode-agent-monitor.clearData', () => {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        const dbPath = path.join(workspaceFolder.uri.fsPath, 'agent_telemetry.db');

        if (fs.existsSync(dbPath)) {
            fs.unlinkSync(dbPath);
            vscode.window.showInformationMessage('Telemetry data cleared');
            if (currentPanel) {
                currentPanel.dispose();
            }
        }
    });

    context.subscriptions.push(showDashboard);
    context.subscriptions.push(showLogs);
    context.subscriptions.push(showEventBrowser);
    context.subscriptions.push(clearData);
}

function showDashboardPanel(dbPath: string, context: vscode.ExtensionContext) {
    // If we already have a panel, show it
    if (currentPanel) {
        currentPanel.reveal(vscode.ViewColumn.One);
        return;
    }

    // Create new panel
    currentPanel = vscode.window.createWebviewPanel(
        'agentMonitorDashboard',
        'Agent Monitor Dashboard',
        vscode.ViewColumn.One,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );

    // Generate and set HTML content using Rust
    try {
        const html = generateDashboardHtmlViaRust(dbPath);
        currentPanel.webview.html = html;
    } catch (error) {
        vscode.window.showErrorMessage(`Failed to generate dashboard: ${error}`);
    }

    // Reset panel when closed
    currentPanel.onDidDispose(() => {
        currentPanel = undefined;
    }, null, context.subscriptions);

    // Add refresh button handler
    currentPanel.webview.onDidReceiveMessage(
        message => {
            if (message.command === 'refresh') {
                try {
                    const html = generateDashboardHtmlViaRust(dbPath);
                    currentPanel!.webview.html = html;
                    vscode.window.showInformationMessage('Dashboard refreshed');
                } catch (error) {
                    vscode.window.showErrorMessage(`Failed to refresh: ${error}`);
                }
            }
        },
        undefined,
        context.subscriptions
    );
}

function generateDashboardHtmlViaRust(dbPath: string): string {
    // Find the extension root directory (where Cargo.toml is)
    // When running in Extension Development Host, __dirname points to the 'out' folder
    const extensionRoot = path.join(__dirname, '..');

    // Check if Cargo.toml exists to verify we have the Rust code
    const cargoTomlPath = path.join(extensionRoot, 'Cargo.toml');
    if (!fs.existsSync(cargoTomlPath)) {
        throw new Error(`Cargo.toml not found at ${cargoTomlPath}. Make sure the Rust code is available.`);
    }

    // Path to the Rust binary
    const cargoBin = 'cargo';
    const tmpHtmlPath = path.join(extensionRoot, '.dashboard-tmp.html');

    // Call Rust to generate HTML
    try {
        vscode.window.showInformationMessage('Generating dashboard...');

        execSync(`${cargoBin} run --example generate_html "${dbPath}" "${tmpHtmlPath}"`, {
            cwd: extensionRoot,
            stdio: 'pipe',
            timeout: 30000 // 30 second timeout
        });

        // Read the generated HTML
        if (!fs.existsSync(tmpHtmlPath)) {
            throw new Error('HTML file was not generated');
        }

        const html = fs.readFileSync(tmpHtmlPath, 'utf-8');

        // Clean up temp file
        fs.unlinkSync(tmpHtmlPath);

        return html;
    } catch (error: any) {
        // Clean up temp file if it exists
        if (fs.existsSync(tmpHtmlPath)) {
            fs.unlinkSync(tmpHtmlPath);
        }
        throw new Error(`Failed to generate HTML via Rust: ${error.message || error}`);
    }
}

export function deactivate() {
    if (captureManager) {
        captureManager.dispose();
    }
}
