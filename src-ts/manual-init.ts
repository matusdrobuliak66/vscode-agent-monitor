// Manual database initialization helper
// This creates the database directly without needing cargo at runtime

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

export function showManualInitInstructions(dbPath: string, extensionRoot: string) {
    const message = `
Database initialization failed.

To manually initialize:
1. Open Terminal
2. Run: cd ${extensionRoot}
3. Run: cargo run --example init_db "${dbPath}"
4. Reload this window

Or click "Open Terminal" and I'll help you.
    `;

    vscode.window.showErrorMessage(
        'Database initialization failed',
        'Open Terminal',
        'Show Instructions'
    ).then(selection => {
        if (selection === 'Open Terminal') {
            const terminal = vscode.window.createTerminal('Agent Monitor Setup');
            terminal.show();
            terminal.sendText(`cd "${extensionRoot}"`);
            terminal.sendText(`cargo run --example init_db "${dbPath}"`);
            vscode.window.showInformationMessage('Run the commands in the terminal, then reload VSCode');
        } else if (selection === 'Show Instructions') {
            vscode.window.showInformationMessage(message);
        }
    });
}
