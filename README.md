# The First Money Markets Protocol on Injective


## iLend contract deployed at 
https://testnet.explorer.injective.network/contract/inj1zhsfj8jzcz30rz678354vafe46vfwlezg9fz7z/


## Main features

- Suppose you have [injectived](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective#install-injectived) installed
- Your INJ address set up `readonly INJ_ADDRESS="YOUR_INJ_ADDRESS"` 
- CONTRACT set up `readonly CONTRACT="inj1zhsfj8jzcz30rz678354vafe46vfwlezg9fz7z"`
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



### Deposit 
Amount that will be deposited to the iLend contract is the amount attached to the call
```
DEPOSIT='{"deposit":{}}'
injectived tx wasm execute $CONTRACT "$DEPOSIT" --from=$(echo $INJ_ADDRESS) --amount="100peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```

### Redeem

```
REDEEM='{"redeem":{"denom":"peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5","amount":"100"}}'
injectived tx wasm execute $CONTRACT "$REDEEM" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```

### Borrow

```
BORROW='{"borrow":{"denom":"peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5","amount":"10000"}}'
injectived tx wasm execute $CONTRACT "$BORROW" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```

### Repay


Amount that will be considered to be a repay amount is the amount attached to the call, the rest of the tokens will be returned back to user

```
REPAY='{"repay":{}}'
injectived tx wasm execute $CONTRACT "$REPAY" --from=$(echo $INJ_ADDRESS) --amount="1000000peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
```

## Tests

run tests in /iLend/contracts/master_contract/ directory
```
cargo wasm && cargo test
```


