import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    // Check if LSP is enabled
    const config = vscode.workspace.getConfiguration('kql');
    const lspEnabled = config.get<boolean>('lsp.enabled', true);
    
    if (!lspEnabled) {
        console.log('KQL LSP is disabled in settings');
        return;
    }

    // Get LSP server path
    let serverPath = config.get<string>('lsp.path', '');
    
    // If no custom path, try to find bundled binary
    if (!serverPath) {
        serverPath = findBundledLsp();
    }

    if (!serverPath || !fs.existsSync(serverPath)) {
        vscode.window.showWarningMessage(
            'KQL LSP server not found. Please install kql-lsp or set kql.lsp.path in settings.'
        );
        return;
    }

    // Server options
    const serverOptions: ServerOptions = {
        run: {
            module: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            module: serverPath,
            transport: TransportKind.stdio,
            options: { execArgv: ['--nolazy', '--inspect=6009'] }
        }
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'kql' },
            { scheme: 'untitled', language: 'kql' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.kql')
        },
        initializationOptions: {
            // Additional options to send to server
        }
    };

    // Create and start client
    client = new LanguageClient(
        'kql',
        'KQL Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client
    const disposable = client.start();
    context.subscriptions.push(disposable);

    // Register commands
    registerCommands(context);

    console.log('KQL Language Server started');
}

function findBundledLsp(): string | undefined {
    // Try to find kql-lsp binary in various locations
    const possiblePaths = [
        // Development: target/debug/kql-lsp
        path.join(__dirname, '..', '..', '..', 'target', 'debug', 'kql-lsp.exe'),
        path.join(__dirname, '..', '..', '..', 'target', 'debug', 'kql-lsp'),
        // Production: relative to extension
        path.join(__dirname, 'kql-lsp.exe'),
        path.join(__dirname, 'kql-lsp'),
        // Cargo bin directory
        path.join(process.env.HOME || '', '.cargo', 'bin', 'kql-lsp.exe'),
        path.join(process.env.HOME || '', '.cargo', 'bin', 'kql-lsp'),
    ];

    for (const p of possiblePaths) {
        if (fs.existsSync(p)) {
            return p;
        }
    }
    return undefined;
}

function registerCommands(context: vscode.ExtensionContext) {
    // Format document command
    context.subscriptions.push(
        vscode.commands.registerCommand('kql.formatDocument', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'kql') {
                await vscode.commands.executeCommand('editor.action.formatDocument');
            }
        })
    );

    // Show syntax tree (placeholder)
    context.subscriptions.push(
        vscode.commands.registerCommand('kql.showSyntaxTree', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'kql') {
                const text = editor.document.getText();
                vscode.window.showInformationMessage(`KQL document length: ${text.length} characters`);
            }
        })
    );

    // Generate SQL (placeholder)
    context.subscriptions.push(
        vscode.commands.registerCommand('kql.generateSQL', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'kql') {
                vscode.window.showInformationMessage('SQL generation would be implemented here');
            }
        })
    );
}

export function deactivate() {
    if (client) {
        return client.stop();
    }
}
