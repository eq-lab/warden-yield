# WardenYield

## Compilation

Install dependencies:
```bash
yarn install
```
Compile solidity files:
```bash
yarn compile
```

## Running tests

Unit tests:
```bash
yarn test
```
Int tests:
```bash
yarn test:int
```

## Deploy

Example of deploy command:
```bash
CREATOR_PRIVATE_KEY=<private_key> \
DEPLOY_DIR=<dir> \
hardhat run tasks/deploy.ts \
  --network holesky
```

To dry-run deploy on a local fork, you need to create and use a hardhat config with `hardhat.forking` settings:
```bash
CREATOR_PRIVATE_KEY=<private_key> \
DEPLOY_DIR=<dir> \
hardhat run tasks/deploy.ts \
  --config holesky-fork.config.ts
```
