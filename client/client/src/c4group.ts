import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { CONFIG_NAME, configKeys } from "./config";

export class C4Group  {
    public static ARG_EXPLODE = "-x";
    public static ARG_PACK = "-p";

    constructor(
        private readonly output: vscode.OutputChannel,
    ) {}

    public unpack(pathToFolder: string): Thenable<void> {
        this.output.appendLine(`Unpacking: ${pathToFolder}`);
        return this.execute([`${pathToFolder}`, C4Group.ARG_EXPLODE]);
    }

    public pack(pathToFolder: string): Thenable<void> {
        this.output.appendLine(`Packing: ${pathToFolder}`);
        return this.execute([`${pathToFolder}`, C4Group.ARG_PACK]);
    }

    private getPathToExecutable() {
        return vscode.workspace.getConfiguration(CONFIG_NAME).get<string>(configKeys.pathToC4gExecutable);
    }

    private execute(args: string[]): Promise<void> {
        const pathToExecutable = this.getPathToExecutable();

        if (!pathToExecutable) {
            vscode.window.showInformationMessage('Path to C4Group executable is not set. Please update your settings.');
            return Promise.resolve();
        }
        
        return new Promise<void>((resolve) => {
            const stdErrChunks = [];

            const cprocess = spawn(pathToExecutable, args);

            this.output.appendLine(pathToExecutable + ' ' + args.join(" "));

            cprocess.stdout.on('data', (data) => {
                this.output.appendLine("STDIN: " + data.toString());
            });

            cprocess.stderr.on('data', (data) => {
                stdErrChunks.push(data);
                this.output.appendLine("STDERR: " + data.toString());
            });

            cprocess.on('error', (err) => {
                const stderr = stdErrChunks.flat().toString();
                this.provideDiagnostics(err, [pathToExecutable, ...args].join(" "), stderr);
            });

            cprocess.once('exit', resolve);
        });
    }

    private loggedKeyError(stdout: string): boolean {
        // This should just be relevant for Clonk Rage
        return stdout.includes("No valid key file found.");
    }

    private provideDiagnostics(err: Error, cmdString: string, stderr: string) {
        const pathToExecutable = this.getPathToExecutable() as string;

        if ('code' in err && err.code === "ENOENT") {
            vscode.window.showErrorMessage(`C4Group executable could not be found at: "${pathToExecutable}". Please check your settings.`);
        } else if (this.loggedKeyError(stderr)) {
            vscode.window.showErrorMessage('Failed to invoke C4Group executable. No key file found.');
        } else {
            this.output.appendLine(`Calling game executable by: ${cmdString}`);
            this.output.appendLine("Error executing c4group: " + err.toString());
        }
    }
}