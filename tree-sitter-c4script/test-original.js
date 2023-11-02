const fs = require('fs');
const path = require('path');
const {execSync, spawnSync} = require('child_process');

const fixturesDir = './test/original_scripts/';

const fnames = fs.readdirSync(fixturesDir);

let successful = 0;

for (const fname of fnames) {
    const fixturePath = path.join(fixturesDir, fname);

    console.log('Checking: ' + fixturePath);

    const r = spawnSync('npx', ['tree-sitter', 'parse', fixturePath]);

    console.log(`Status: ${r.status}; Error: ${r.error?.toString() ?? 'none'}`);
    
    if (r.status === 0) {
        successful++;
    }
}

console.log(`From ${fnames.length} total, ${successful} were successfully parsed`);
