# CosmWasm part of Warden Yield

## Contracts

All the contracts and their descriptions can be found in `src` directory

## Deployments

TODO

## Development

### Installing

Installation is local and doesn't require root privileges.

Use rustc 1.79.0 version and add `wasm32-unknown-unknown` target:

```bash
rustup default 1.79.0
rustup target add wasm32-unknown-unknown
```

### Build & test

Compile wasm contract:

```bash
cargo wasm
```

Run unit tests:

```bash
cargo unit-test
```

Test coverage:

```bash
cargo tarpaulin
```

If `tarpaulin` is missing, then

```bash
cargo install cargo-tarpaulin
```

### Deploying

TODO

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
