import * as path from 'path';
import { commands, ExtensionContext, OutputChannel, window } from 'vscode';

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
	bindUi(context, outputChannel);

	const executableName = process.platform === 'win32' ? 'legacy-clonk-ls.exe' : 'legacy-clonk-ls';

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
			command: '/home/fmi/source/rust/cptee/target/debug/cptee',
			args: [pathToBinInDebug],
			transport: TransportKind.stdio,
		},
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{
			scheme: 'file',
			language: 'c4script',
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

function bindUi(context: ExtensionContext, outputChannel: OutputChannel) {
	const c4group = new C4Group(outputChannel);
	const runner = new ScenarioRunner();

	context.subscriptions.push(commands.registerCommand(CONFIG_NAME + '.unpackC4g', ({ fsPath }) => {
		c4group.unpack(fsPath)
			.then(_ => commands.executeCommand("workbench.files.action.refreshFilesExplorer"));
	}));

	context.subscriptions.push(commands.registerCommand(CONFIG_NAME + '.packC4g', ({ fsPath, ...args }) => {
		c4group.pack(fsPath)
			.then(_ => commands.executeCommand("workbench.files.action.refreshFilesExplorer"));
	}));

	context.subscriptions.push(commands.registerCommand(CONFIG_NAME + '.runScenarioInEditor', ({ fsPath }) => {
		runner.run(fsPath, outputChannel);
	}));
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
