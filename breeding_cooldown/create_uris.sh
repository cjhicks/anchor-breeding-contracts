#!/bin/bash
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="/Users/chicks1024/.config/solana/devnet.json"

yarn run ts-mocha -p ./tsconfig.json -t 1000000 scripts/create_uris.ts
