# CLI tools

Build:

```bash
cargo build
```

Encode stake message:

```bash
../target/debug/tools stake --token "weth"
```

Encode unstake message:

```bash
../target/debug/tools unstake --token "weth"
```

Encode stake response message:

```bash
../target/debug/tools stake-response \
    --token "weth" \
    --stake-id 1 \
    --is-success \
    --reinit-unstake-id 0 \
    --lp-token-amount 101 \
    --return-amount 0
```

Encode unstake response message:

```bash
../target/debug/tools unstake-response \
    --token "weth" \
    --unstake-id 1 \
    --is-success \
    --reinit-unstake-id 1 \
    --return-amount 111
```

```bash
wardend tx wasm execute \  
  warden1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qvhthj6 \  
  '{"send":{"contract":"warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32","amount":"111","msg":"eyJoYW5kbGVfcmVzcG9uc2UiOnsiZGVwb3NpdF90b2tlbl9kZW5vbSI6ImRlbW9fd2V0aCIsInNvdXJjZV9jaGFpbiI6IkV0aGVyZXVtIiwic291cmNlX2FkZHJlc3MiOiIweDRERjY2QkNBOTYzMTlDNkEwMzNjZmQ4NmMzOEJDRGI5QjNjMTFhNzIiLCJwYXlsb2FkIjoiQVFBQUFBQUFBQUFBQVFBQUFBQUFBQUFCIn19"}}' \  
  --from "keplr_test2_acc" -y \  
  --node "https://rpc.buenavista.wardenprotocol.org:443" \  
  --chain-id "buenavista-1" \
  --gas auto --gas-adjustment 1.3
```

## Stake messages

| Token | Message                                                                                                                                                                           |
|-------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| WETH  | `{"send":{"contract":"warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32","amount":"12345","msg":"eyJzdGFrZSI6eyJkZXBvc2l0X3Rva2VuX2Rlbm9tIjoiZGVtb193ZXRoIn19"}}` |
| USDT  | `{"send":{"contract":"warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32","amount":"12345","msg":"eyJzdGFrZSI6eyJkZXBvc2l0X3Rva2VuX2Rlbm9tIjoiZGVtb191c2R0In19"}}` |
| USDC  | `{"send":{"contract":"warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32","amount":"12345","msg":"eyJzdGFrZSI6eyJkZXBvc2l0X3Rva2VuX2Rlbm9tIjoiZGVtb191c2RjIn19"}}` |

## Unstake messages

| Token | Message                                                                                                                                                                             |
|-------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| WETH  | `{"send":{"contract":"warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32","amount":"123","msg":"eyJ1bnN0YWtlIjp7ImRlcG9zaXRfdG9rZW5fZGVub20iOiJkZW1vX3dldGgifX0="}}` |
| USDT  | `{"send":{"contract":"warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32","amount":"123","msg":"eyJ1bnN0YWtlIjp7ImRlcG9zaXRfdG9rZW5fZGVub20iOiJkZW1vX3VzZHQifX0="}}` |
| USDC  | `{"send":{"contract":"warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32","amount":"123","msg":"eyJ1bnN0YWtlIjp7ImRlcG9zaXRfdG9rZW5fZGVub20iOiJkZW1vX3VzZGMifX0="}}` |