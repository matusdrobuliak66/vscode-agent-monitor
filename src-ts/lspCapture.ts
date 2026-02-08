// LSP Capture Module - Simplified approach
// Monitors document changes and provides manual capture

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { execSync } from 'child_process';
import { showManualInitInstructions } from './manual-init';

export class LSPCaptureManager {
    private dbPath: string;
    private extensionRoot: string;
    private disposables: vscode.Disposable[] = [];
    private outputChannel: vscode.OutputChannel;
    private captureCount: number = 0;

    constructor(workspaceFolder: string, extensionRoot: string) {
        this.dbPath = path.join(workspaceFolder, 'agent_telemetry.db');
        this.extensionRoot = extensionRoot;
        this.outputChannel = vscode.window.createOutputChannel('Agent Monitor');

        this.outputChannel.appendLine('🔍 Agent Monitor: Initializing...');
        this.outputChannel.appendLine(`Database path: ${this.dbPath}`);

        this.initializeDatabase();
    }

    private initializeDatabase() {
        if (fs.existsSync(this.dbPath)) {
            this.outputChannel.appendLine('✅ Database already exists');
            return;
        }

        this.outputChannel.appendLine('📊 Creating database...');

        try {
            // Use the simpler init_db example
            execSync(`cargo run --example init_db "${this.dbPath}"`, {
                cwd: this.extensionRoot,
                stdio: 'pipe',
                timeout: 15000
            });

            if (fs.existsSync(this.dbPath)) {
                this.outputChannel.appendLine('✅ Database created successfully');
                vscode.window.showInformationMessage('Agent Monitor: Database initialized');
            } else {
                throw new Error('Database file was not created');
            }
        } catch (error: any) {
            this.outputChannel.appendLine(`❌ Database initialization failed: ${error.message}`);
            this.outputChannel.appendLine(`Full error: ${JSON.stringify(error, null, 2)}`);

            // Check if it's a cargo not found error
            if (error.message && error.message.includes('cargo')) {
                this.outputChannel.appendLine(`💡 Cargo not found in PATH. Cargo location: ${process.env.PATH}`);
            }

            showManualInitInstructions(this.dbPath, this.extensionRoot);
        }
    }

    public startCapturing() {
        this.outputChannel.appendLine('👀 Starting to monitor document changes...');

        // Monitor document changes (includes Copilot acceptances)
        const changeDisposable = vscode.workspace.onDidChangeTextDocument(event => {
            if (event.contentChanges.length > 0) {
                this.captureDocumentChange(event);
            }
        });

        // Monitor when documents are saved
        const saveDisposable = vscode.workspace.onDidSaveTextDocument(document => {
            this.captureDocumentSave(document);
        });

        this.disposables.push(changeDisposable, saveDisposable);

        vscode.window.showInformationMessage(
            'Agent Monitor: Monitoring active. Use "Agent Monitor: Show Logs" to see activity.',
            'Show Logs'
        ).then(selection => {
            if (selection === 'Show Logs') {
                this.outputChannel.show();
            }
        });
    }

    private async captureDocumentChange(event: vscode.TextDocumentChangeEvent) {
        const document = event.document;

        // Skip if not a real file
        if (document.uri.scheme !== 'file') {
            return;
        }

        // Capture significant changes (more than a few characters)
        const totalChanges = event.contentChanges.reduce((sum, change) => sum + change.text.length, 0);

        if (totalChanges > 10) { // Only capture substantial changes (likely Copilot completions)
            this.outputChannel.appendLine(`📝 Change detected in ${path.basename(document.fileName)} (+${totalChanges} chars)`);

            // Get context around the change
            const change = event.contentChanges[0];
            const startLine = Math.max(0, change.range.start.line - 5);
            const endLine = Math.min(document.lineCount - 1, change.range.end.line + 5);
            const context = document.getText(new vscode.Range(startLine, 0, endLine, 1000));

            await this.logInteraction('textDocument/didChange', {
                uri: document.uri.toString(),
                language: document.languageId,
                changeSize: totalChanges,
                context: context
            });
        }
    }

    private async captureDocumentSave(document: vscode.TextDocument) {
        if (document.uri.scheme !== 'file') {
            return;
        }

        this.outputChannel.appendLine(`💾 Document saved: ${path.basename(document.fileName)}`);

        // Get full document content on save
        const content = document.getText();

        await this.logInteraction('textDocument/didSave', {
            uri: document.uri.toString(),
            language: document.languageId,
            contentLength: content.length,
            preview: content.substring(0, 500) // First 500 chars
        });
    }

    private async logInteraction(method: string, data: any) {
        if (!fs.existsSync(this.dbPath)) {
            this.outputChannel.appendLine('⚠️  Database not found, skipping capture');
            return;
        }

        try {
            const requestData = JSON.stringify(data, null, 2);

            // Create a temp file with the data
            const tmpDataFile = path.join(this.extensionRoot, `.telemetry_${Date.now()}.json`);
            fs.writeFileSync(tmpDataFile, JSON.stringify({
                method: method,
                request: requestData,
                response: null
            }));

            // Call Rust to store it
            execSync(`cargo run --example store_telemetry "${this.dbPath}" "${tmpDataFile}"`, {
                cwd: this.extensionRoot,
                stdio: 'pipe',
                timeout: 5000
            });

            // Clean up
            if (fs.existsSync(tmpDataFile)) {
                fs.unlinkSync(tmpDataFile);
            }

            this.captureCount++;
            this.outputChannel.appendLine(`✅ Captured (${this.captureCount} total)`);
        } catch (error: any) {
            this.outputChannel.appendLine(`❌ Failed to log: ${error.message}`);
        }
    }

    public showLogs() {
        this.outputChannel.show();
    }

    public getCaptureCount(): number {
        return this.captureCount;
    }

    public dispose() {
        this.disposables.forEach(d => d.dispose());
        this.disposables = [];
        this.outputChannel.dispose();
    }
}
