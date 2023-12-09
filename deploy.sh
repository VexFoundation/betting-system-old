#!/bin/sh

./build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying contract"

# https://docs.near.org/tools/near-cli#near-dev-deploy
npx near-cli dev-deploy ./target/wasm32-unknown-unknown/release/betting_system.wasm
# near deploy --accountId contractholder.testnet  --wasmFile ./target/wasm32-unknown-unknown/release/betting_system.wasm