# This script does not install any of the necessary dependencies for
# building individual artifacts

set -x xtrace
set -e

# Step 1: build grammer

cd tree-sitter-c4script
npm run build
cd ..

# Step 2: build language server for all platforms
cd server
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-unknown-linux-gnu
cd ..

# Step 3: build vscode extension
cd client
npm run compile

# Step 4: Move server artifacts to vscode extension package directory
rm -f client/out/legacy-clonk-ls
rm -f client/out/legacy-clonk-ls.exe
mv ../server/target/x86_64-pc-windows-gnu/release/legacy-clonk-ls.exe client/out/
mv ../server/target/x86_64-unknown-linux-gnu/release/legacy-clonk-ls client/out/

# Step 5: Packaging and publishing
# But these commands should better be isssued by hand:
# vsce package
# vsce publish
