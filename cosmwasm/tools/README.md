# CLI tools

Build:

```bash
cargo build
```

Encode stake 15 USDC message:

```bash
../target/debug/tools stake --token "usdc" --amount 15000000
```

Encode unstake 0.001 LPETH message:

```bash
../target/debug/tools unstake --token "weth" --amount 1000000000000
```

Encode stake response message:

```bash
../target/debug/tools stake-response \
    --token "weth" \
    --stake-id 1 \
    --is-success \
    --reinit-unstake-id 0 \
    --lp-token-amount 10000000000000000 \
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