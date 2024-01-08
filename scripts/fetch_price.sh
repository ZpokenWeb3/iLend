# get the contract address from explorer tx of instantiating
CONTRACT="inj1qunaqndmy45x6sa0uht09ve0n4a8cuvmsmqfm0"
readonly INJ_ADDRESS="inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z"

#SET_PYTH_ADDR='{"set_pyth_contract": {"pyth_contract_addr": "inj18rlflp3735h25jmjx97d22c72sxk260amdjxlu"}}'
#injectived tx wasm execute $CONTRACT "$SET_PYTH_ADDR" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443

wait

#UPDATE_PRICES='{"update_price": {"denom": "inj", "price": "0"}}'
#yes 12345678 | injectived tx wasm execute $CONTRACT "$UPDATE_PRICES" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443



wait

FETCH_PRICE_INJ='{"get_price": {"denom": "inj"}}'
FETCH_PRICE_APE='{"get_price": {"denom": "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7"}}'
FETCH_PRICE_USDT='{"get_price": {"denom": "peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5"}}'

injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_INJ" --node=https://k8s.testnet.tm.injective.network:443 --output json
injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_APE" --node=https://k8s.testnet.tm.injective.network:443 --output json
injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_USDT" --node=https://k8s.testnet.tm.injective.network:443 --output json
