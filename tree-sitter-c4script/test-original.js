const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const fixturesDir = './test/original_scripts/';

const fnames = fs.readdirSync(fixturesDir).slice();

let successful = 0;

function spawnAsync(cmd, args) {
    return new Promise((res, rej) => {
        const cp = spawn(cmd, args, {
            stdio: 'ignore',
        });

        cp.once('error', rej);
        cp.once('exit', (code, _sig) => res(code));
    });
}

Promise.allSettled(fnames.map(async fname => {
    const fixturePath = path.join(fixturesDir, fname);

    let log = 'Checking: ' + fixturePath;

    let code;
    try {
        code = await spawnAsync('npx', ['tree-sitter', 'parse', fixturePath]);
    } catch (e) {
        console.error(`Error parsing '${fixturePath}': ${e}`);
        return -1;
    }

    log += `Status: ${code}`;
    
    if (code === 0) {
        successful++;
    }

    return log;
})).then(results => {

    results.forEach(p => {
        console.log(p.value);
    });
    
    console.log(`From ${fnames.length} total, ${successful} were successfully parsed`);

});
