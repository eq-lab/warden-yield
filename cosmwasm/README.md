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
  --from "keplr_test2_acc" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3 \
  | wardend q wait-tx --node "https://rpc.buenavista.wardenprotocol.org:443" -o json | jq '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value | tonumber'
```

## Store YieldWard code

```bash
wardend tx wasm store yield_ward.wasm \
  --from "keplr_test2_acc" -y \
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
wardend tx wasm instantiate 5 \
  '{"tokens": [], "axelar": "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl", "lp_token_code_id": 3}' \
  --label "YieldWard-2024-08-21" \
  --admin "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl" \
  --from "keplr_test2_acc" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1"
```

## Instantiate CW-20 contract

```bash
wardend tx wasm instantiate 3 \
  '{"name":"Ethereum","symbol":"ETH","decimals":18,"initial_balances":[],"mint":{"minter":"warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl"}}' \
  --label "ETH-demo" \
  --admin "warden1ptf722r8tnxk8c7w5twln6xxp9hzc2fzr5djkl" \
  --from "keplr_test2_acc" -y \
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

TODO

## Mint CW-20

```bash  
wardend tx wasm execute \
  warden1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qvhthj6 \
  '{"mint":{"recipient":"warden1sl8gckyk7ya5cjmsj8gxu857jt7amxqly4wvdu","amount":"1000000000000000000"}}' \
  --from "keplr_test2_acc" -y \
  --node "https://rpc.buenavista.wardenprotocol.org:443" \
  --chain-id "buenavista-1"
```

## Read CW-20 balance

```bash
wardend query wasm contract-state smart \
  warden1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qvhthj6 \
  '{"balance":{"address":"warden1sl8gckyk7ya5cjmsj8gxu857jt7amxqly4wvdu"}}' \
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