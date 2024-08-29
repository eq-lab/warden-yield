# Warden Yield

## Contracts

All the contracts and their descriptions can be found in `contracts` directory

## Deployments

All deployed contracts addresses can be found in [deploy/data/contracts](./deploy/data/contracts)

## Development

### Installing

Installation is local and doesn't require root privileges.

If you have `yarn` installed globally:

```bash
yarn
```

otherwise:

```bash
npx yarn
```

### Build & test

Run unit tests:

```bash
yarn test
```

Run integration tests:
```bash
yarn test:int
```

Run unit tests and report gas used by each Solidity function:

```bash
yarn test:gas
```

Generate unit test coverage report:

```bash
yarn test:coverage
```

### Deploying

Example of deploy command:
```bash
hardhat --network holesky \
  task:deploy \
  --network-name <network> \
  --creator-key <private_key> \
  --is-private-key
```

To dry-run deploy on a local fork, you need to create and use a hardhat config with `hardhat.forking` settings:
```bash
hardhat --config holesky-fork.config.ts \
  task:deploy \
  --network-name <network> \
  --creator-key <private_key>
  --is-private-key
```

You can use impersonated signer while forking:
```bash
hardhat --config holesky-fork.config.ts \
  task:deploy \
  --network-name <network> \
  --creator-key <public_key>
```

### Upgrading
For upgrade to work there must be `./openzeppelin/<network>.json` manifest file for `openzeppelin-upgrades` checks and `deploy/data/contracts/<network>.json` file to get contract address from

Example of upgrade command:
```bash
hardhat --network holesky \
  task:upgrade \
  --network-name <network> \
  --creator-key <private_key> \
  --is-private-key
```

To dry-run upgrade on a local fork, you need to create and use a hardhat config with `hardhat.forking` settings:
```bash
hardhat --config holesky-fork.config.ts \
  task:upgrade \
  --network-name <network> \
  --creator-key <private_key>
  --is-private-key
```

You can use impersonated signer while forking:
```bash
hardhat --config holesky-fork.config.ts \
  task:upgrade \
  --network-name <network> \
  --creator-key <public_key>
```

# License

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the [GNU General Public License](LICENSE)
along with this program. If not, see <https://www.gnu.org/licenses/>.
