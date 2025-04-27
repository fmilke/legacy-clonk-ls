import * as path from 'path';
import { commands, ExtensionContext, extensions, OutputChannel, QuickPickItem, window } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';
import { C4Group } from './c4group';
import { ScenarioRunner } from './runner';
import { CONFIG_NAME } from './config';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
	const outputChannel = window.createOutputChannel('Legacy Clonk');
	configureTreeViewCommands(context, outputChannel);
	configureIdSelectCommand(context, outputChannel);
	configureAndStartLanguageServer(context);
}

function configureAndStartLanguageServer(context: ExtensionContext) {

	const executableName = process.platform === 'win32' ? 'lsp.exe' : 'lsp';

	const pathToBin = context.asAbsolutePath(
		path.join('client', 'out', executableName)
	);

	const pathToBinInDebug = context.asAbsolutePath(
		path.join('..', 'server', 'target', 'debug', executableName)
	);

	const serverOptions: ServerOptions = {
		run: {
			command: pathToBin,
			args: [],
			transport: TransportKind.stdio,
		},
		debug: {
			command: pathToBinInDebug,
			args: ["--debug"],
			transport: TransportKind.stdio,
		},
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{
			scheme: 'file',
			language: 'c4script',
		}, {
			scheme: 'file',
			language: 'c4ini',
		}],
	};

	client = new LanguageClient(
		'legacyClonkLanguageServer',
		'Legacy Clonk Language Server',
		serverOptions,
		clientOptions
	);

	client.start();
	client.info("Client started");
}

function configureTreeViewCommands(context: ExtensionContext, outputChannel: OutputChannel) {
	const c4group = new C4Group(outputChannel);
	const runner = new ScenarioRunner();

	context.subscriptions.push(commands.registerCommand(CONFIG_NAME + '.unpackC4g', ({ fsPath }) => {
		c4group.unpack(fsPath)
			.then(_ => commands.executeCommand("workbench.files.action.refreshFilesExplorer"));
	}));

	context.subscriptions.push(commands.registerCommand(CONFIG_NAME + '.packC4g', ({ fsPath }) => {
		c4group.pack(fsPath)
			.then(_ => commands.executeCommand("workbench.files.action.refreshFilesExplorer"));
	}));

	context.subscriptions.push(commands.registerCommand(CONFIG_NAME + '.runScenarioInEditor', ({ fsPath }) => {
		runner.run(fsPath, outputChannel);
	}));
}

function configureIdSelectCommand(context: ExtensionContext, outputChannel: OutputChannel) {
	context.subscriptions.push(commands.registerCommand(CONFIG_NAME + '.pasteId', () => {

		client.sendRequest('x-legacy-clonk/getAllIds')
			.then(response => {

				if (!Array.isArray(response)) {
					outputChannel.appendLine(`Value returned from language server was expected to be an array but was ${typeof response}`);
					return;
				}

				outputChannel.appendLine(`x-legacy-clonk/getAllIds returned ${response.length} results`);

				const items: QuickPickItem[] = [];

				// Validate?
				for (const v of response) {
					items.push({
						label: v.id,
						detail: v.tlc,
						description: v.name,
					});
				}

				return window.showQuickPick(items, {
					title: 'this is title',
					matchOnDescription: true,
				});
			}).then(item => {
				if (typeof item === 'object') {
					return window.activeTextEditor.edit(b => {
						b.replace(window.activeTextEditor.selection, item.label);
					});
				}
			});
	}));
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
