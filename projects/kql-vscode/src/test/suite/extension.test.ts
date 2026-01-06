import * as assert from 'assert';
import * as vscode from 'vscode';

suite('Extension Test Suite', () => {
    test('Extension should be active', async () => {
        const extension = vscode.extensions.getExtension('kql-team.kql-vscode');
        assert.ok(extension, 'Extension not found');
        
        // Wait for extension to activate
        await extension.activate();
        assert.ok(extension.isActive, 'Extension not active');
    });

    test('KQL language should be registered', async () => {
        const languages = await vscode.languages.getLanguages();
        assert.ok(languages.includes('kql'), 'KQL language not registered');
    });
});
