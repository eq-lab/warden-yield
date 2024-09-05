# CosmWasm contracts

## Deploy demo steps

1. Store CW20 token code
2. Store YieldWard code
3. Instantiate CW20 deposit token contracts: ETH, USDT, USDC
4. Instantiate YieldWard contract
5. Call YieldWard::AddToken
6. Mint CW20 deposit tokens to Axelar account

## Store CW20 token code

```bash
wardend tx wasm store lp_token.wasm \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3 \
  | wardend q wait-tx --node "https://rpc.buenavista.wardenprotocol.org:443" -o json | jq '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value | tonumber'
```

## Store YieldWard code

```bash
wardend tx wasm store yield_ward.wasm \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3 \
  | wardend q wait-tx --node "https://rpc.buenavista.wardenprotocol.org:443" -o json | jq '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value | tonumber'
```

## Get list of codes

```bash
wardend query wasm list-code \
  --node https://rpc.buenavista.wardenprotocol.org:443
```

## Instantiate YieldWard contract

```bash
wardend tx wasm instantiate 8 \
  '{"axelar": "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl", "lp_token_code_id": 9}' \
  --label "YieldWard-2024-08-29" \
  --admin "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl" \
  --from "keplr_test2_acc" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

## Instantiate CW-20 contract

```bash
wardend tx wasm instantiate 9 \
  '{"name":"Ethereum","symbol":"ETH","decimals":18,"initial_balances":[],"mint":{"minter":"warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl"}}' \
  --label "ETH-demo-2024-08-29" \
  --admin "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl" \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1"

wardend tx wasm instantiate 9 \
  '{"name":"Tether USD","symbol":"USDT","decimals":6,"initial_balances":[],"mint":{"minter":"warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl"}}' \
  --label "USDT-demo-2024-08-29" \
  --admin "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl" \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1"

wardend tx wasm instantiate 9 \
  '{"name":"Circle USD","symbol":"USDC","decimals":6,"initial_balances":[],"mint":{"minter":"warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl"}}' \
  --label "USDC-demo-2024-08-29" \
  --admin "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl" \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1"
```

## Get list of deployed contracts

```bash
wardend query wasm list-contract-by-code 5 \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --output json
```

## Listen tx events

```bash
wardend q wait-tx --node "https://rpc.buenavista.wardenprotocol.org:443" \
  -o json \
  | jq '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value | tonumber'
```

## Add token

WETH:

```bash  
wardend tx wasm execute \
  warden1vhjnzk9ly03dugffvzfcwgry4dgc8x0sv0nqqtfxj3ajn7rn5ghq6xwfv9 \
  '{"add_token":{"token_denom":"demo_weth","cw20_address":"warden1dk9c86h7gmvuaq89cv72cjhq4c97r2wgl5gyfruv6shquwspalgqr09xey","is_stake_enabled":true,"is_unstake_enabled":true,"chain":"Ethereum","lpt_symbol":"LPWETH","lpt_name":"Wrapped Ether LPT","evm_yield_contract":"0x4DF66BCA96319C6A033cfd86c38BCDb9B3c11a72","evm_address":"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2","lp_token_denom":"demo_weth_lp"}}' \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

USDT:

```bash
wardend tx wasm execute \
  warden1vhjnzk9ly03dugffvzfcwgry4dgc8x0sv0nqqtfxj3ajn7rn5ghq6xwfv9 \
  '{"add_token":{"token_denom":"demo_usdt","cw20_address":"warden1xqkp8x4gqwjnhemtemc5dqhwll6w6rrgpywvhka7sh8vz8swul9ss7ztuk","is_stake_enabled":true,"is_unstake_enabled":true,"chain":"Ethereum","lpt_symbol":"LPUSDT","lpt_name":"Tether USD LPT","evm_yield_contract":"0x0F9d2C03AD21a30746A4b4f07919e1C5F3641F35","evm_address":"0xdAC17F958D2ee523a2206206994597C13D831ec7","lp_token_denom":"demo_usdt_lp"}}' \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

USDC:

```bash
wardend tx wasm execute \
  warden1vhjnzk9ly03dugffvzfcwgry4dgc8x0sv0nqqtfxj3ajn7rn5ghq6xwfv9 \
  '{"add_token":{"token_denom":"demo_usdc","cw20_address":"warden1euqmngymytlt8j707spv9hn6ajzy92ndfjk47pnlu9uzmfuyplhsyeh0vc","is_stake_enabled":true,"is_unstake_enabled":true,"chain":"Ethereum","lpt_symbol":"LPUSDC","lpt_name":"Circle USD LPT","evm_yield_contract":"0x0259044395FE54d8aFe28354Ac737EB216064cF9","evm_address":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","lp_token_denom":"demo_usdc_lp"}}' \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

## Mint CW-20

ETH:

```bash  
wardend tx wasm execute \
  warden1dk9c86h7gmvuaq89cv72cjhq4c97r2wgl5gyfruv6shquwspalgqr09xey \
  '{"mint":{"recipient":"warden1730hj2t4dycp7aa2uv63gsuuhrnc5x50x70wkf","amount":"100000000000000000"}}' \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

USDT:

```bash  
wardend tx wasm execute \
  warden1xqkp8x4gqwjnhemtemc5dqhwll6w6rrgpywvhka7sh8vz8swul9ss7ztuk \
  '{"mint":{"recipient":"warden1730hj2t4dycp7aa2uv63gsuuhrnc5x50x70wkf","amount":"1000000000"}}' \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

USDC:

```bash  
wardend tx wasm execute \
  warden1euqmngymytlt8j707spv9hn6ajzy92ndfjk47pnlu9uzmfuyplhsyeh0vc \
  '{"mint":{"recipient":"warden1mscv9y928gq6ankczrwr8mgt2fc6matv0086dy","amount":"1000000000"}}' \
  --from "yieldward-stage-admin" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

## Read CW-20 balance

```bash
wardend query wasm contract-state smart \
  warden1dk9c86h7gmvuaq89cv72cjhq4c97r2wgl5gyfruv6shquwspalgqr09xey \
  '{"balance":{"address":"warden1mscv9y928gq6ankczrwr8mgt2fc6matv0086dy"}}' \
  --node "https://rpc.buenavista.wardenprotocol.org:443"
```

## Read CW-20 metadata

```bash

wardend query wasm contract-state smart \
  warden1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qvhthj6 \
  '{"token_info":{}}' \
  --node "https://rpc.buenavista.wardenprotocol.org:443"
```

## Read YieldWard config

```bash
wardend query wasm contract-state smart \
  warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32 \
  '{"contract_config":{}}' \
  --node "https://rpc.buenavista.wardenprotocol.org:443"
```