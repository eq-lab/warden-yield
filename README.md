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
hardhat --network holesky \
  task:deploy \
  --deploy-dir "./data/deploy/holesky" \
  --creator-private-key <private_key>
```
To dry-run deploy on a local fork, you need to create and use a hardhat config with `hardhat.forking` settings:
```bash
hardhat --config holesky-fork.config.ts \
  task:deploy \
  --deploy-dir "./data/deploy/holesky" \
  --creator-private-key <private_key>
```