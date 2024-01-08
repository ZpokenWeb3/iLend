readonly CONTRACT="inj1qunaqndmy45x6sa0uht09ve0n4a8cuvmsmqfm0"
readonly INJ_ADDRESS="inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z"

sleep 1

DEPOSIT='{"deposit":{}}'
# shellcheck disable=SC2046
# shellcheck disable=SC2116
yes 12345678 | injectived tx wasm execute $CONTRACT "$DEPOSIT" --from=$(echo $INJ_ADDRESS) --amount="100peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443

sleep 2

# query balance after deposit
BALANCE_QUERY='{"get_deposit": {"address": "'$(echo $INJ_ADDRESS)'", "denom": "peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5"}}'
injectived query wasm contract-state smart $CONTRACT "$BALANCE_QUERY" --node=https://k8s.testnet.tm.injective.network:443 --output json

sleep 2

REDEEM='{"redeem":{"denom":"peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5","amount":"100"}}'
# shellcheck disable=SC2046
# shellcheck disable=SC2116
yes 12345678 | injectived tx wasm execute $CONTRACT "$REDEEM" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443

sleep 2

# query balance after redeeming
BALANCE_QUERY='{"get_deposit": {"address": "'$(echo $INJ_ADDRESS)'", "denom": "peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5"}}'
injectived query wasm contract-state smart $CONTRACT "$BALANCE_QUERY" --node=https://k8s.testnet.tm.injective.network:443 --output json
