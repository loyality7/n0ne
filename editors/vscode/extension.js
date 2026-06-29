const vscode = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function activate(context) {
    const serverOptions = {
        run: { command: 'n0ne', args: ['lsp'] },
        debug: { command: 'n0ne', args: ['lsp'] }
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'n0ne' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.n0')
        }
    };

    client = new LanguageClient(
        'n0neLsp',
        'n0ne Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.deactivate();
}

module.exports = {
    activate,
    deactivate
};
