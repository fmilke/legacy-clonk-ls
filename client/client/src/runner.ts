import { spawn } from 'child_process';
import { OutputChannel, window, workspace } from "vscode";
import { CONFIG_NAME } from './config';

export class ScenarioRunner {

    public static readonly FLAG_DEV_MODE = "/console";
    
    public run(pathToScenario: string, outputChannel: OutputChannel) {
        const pathToExecutable = this.getPathToGameExecutable();

        if (!pathToExecutable) {
            window.showInformationMessage('Path to Legacy Clonk executable is not set. Please update your settings.');
            return;
        }

        this.execute([`${pathToScenario}`, ScenarioRunner.FLAG_DEV_MODE], outputChannel);
    }
n
    private execute(args: string[], outputChannel: OutputChannel) {
        const pathToExecutable = this.getPathToGameExecutable();
        
        if (!pathToExecutable) {
            window.showInformationMessage('Path to Legacy Clonk executable is not set. Please update your settings.');
            return;
        }

        const cprocess = spawn(pathToExecutable, args);

        cprocess.stdout.on('data', function (data) {
            outputChannel.append(data.toString());
        });

        cprocess.stderr.on('data', function (data) {
            outputChannel.append(data.toString());
        });

        cprocess.on('error', (err) => {
            this.provideDiagnostics(err, [pathToExecutable, ...args].join(" "));
        });
    }

    private provideDiagnostics(err: Error, cmdString: string) {
        const pathToExecutable = this.getPathToGameExecutable() as string;

        if ('code' in err && err.code === "ENOENT") {
            window.showErrorMessage(`Legacy Clonk executable could not be found at: "${pathToExecutable}". Please check your settings.`);
        }
        else {
            console.log(`Calling game executable by: ${cmdString}`);
            console.error(err);
        }
    }

    private getPathToGameExecutable() {
        return workspace.getConfiguration(CONFIG_NAME).get<string>("pathToGameExecutable");
    }
}