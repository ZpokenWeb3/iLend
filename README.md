# Overview

## What is i-Lend Protocol?
i-Lend is a decentralized finance protocol developed on the Injective network. It offers the ability to engage in lending and borrowing activities in a decentralized, transparent, and efficient manner. The protocol leverages Injective Protocol's ability to offer fast, secure, and EVM-compatible DeFi transactions across multiple blockchain ecosystems


## Contracts




|                                                                     | contract's link                                                                                |
|----------------------|------------------------------------------------------------------------------------------------|
| master contract | https://testnet.explorer.injective.network/contract/inj1ulkyckufg8f0q20nsavcq5shcttq0n8nlc39t4/ |





## Prerequisites


To run the project you need:

 - Rustup, Rustc  and Cargo installed. Check the detailed information [here](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective#prerequisites)
 - [injectived](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective#install-injectived) command-line interface installed. This enables to interact with the Injective blockchain.
- You can mint yourself a test tokens [here](https://testnet.faucet.injective.network/)


### Supported tokens 

```
{"data":
       {"supported_tokens":
            [
                {"denom":"inj","name":"Injective","symbol":"INJ","decimals":"18"},
                {"denom":"peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7","name":"Ape Coin","symbol":"APE","decimals":"18"},
                {"denom":"peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5","name":"Tether","symbol":"USDT","decimals":"6"}
            ]
       }
}
```

## Core functionalities

1) Deposit: Users can safely store their assets into the i-Lend protocol. Once deposited, these assets immediately start accruing interest, allowing users to grow their holdings over time.
2) Redeem: i-Lend allows for easy and convenient withdrawal of assets, including any accrued interest, ensuring users have constant access to their funds.
3) Borrow: Users can leverage their deposited assets as collateral to secure loans. This provides an efficient method to access additional funds without needing to liquidate existing holdings.
4) Repay: i-Lend facilitates seamless repayment of borrowed assets. On completion of repayment, the accumulated interest is settled, reducing potential risk against the user's collateral.


## CLI commands (For Developers and Advanced Users)

Make sure you have your injective address and i-lend contract set up
- `readonly INJ_ADDRESS="YOUR_INJ_ADDRESS"`
- `readonly CONTRACT="RESPECTIVE_CONTRACT_ADDR_FROM_TABLE_ABOVE"`
- you can find respective supported tokens information above or via command
```
  GET_SUPPORTED_TOKENS_QUERY='{"get_supported_tokens": {}}'
  injectived query wasm contract-state smart $CONTRACT "$GET_SUPPORTED_TOKENS_QUERY" --node=https://k8s.testnet.tm.injective.network:443 --output json
```


### DEPOSIT 

amount attached to the call will be considered the deposited amount

```
DEPOSIT='{"deposit":{}}'
injectived tx wasm execute $CONTRACT "$DEPOSIT" --from=$(echo $INJ_ADDRESS) --amount="100peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```


### REDEEM

```
REDEEM='{"redeem":{"denom":"peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5","amount":"100"}}'
injectived tx wasm execute $CONTRACT "$REDEEM" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```


### BORROW
```
BORROW='{"borrow":{"denom":"peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5","amount":"10000"}}'
injectived tx wasm execute $CONTRACT "$BORROW" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```

### REPAY
 amount attached to the call will be considered to be a repay amount, remaining tokens will be refunded to user

```
REPAY='{"repay":{}}'
injectived tx wasm execute $CONTRACT "$REPAY" --from=$(echo $INJ_ADDRESS) --amount="1000000peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```

## Tests

run tests in /iLend/contracts/master_contract/ directory
```
cargo wasm && cargo test
```


