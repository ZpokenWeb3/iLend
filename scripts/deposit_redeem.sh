readonly CONTRACT="inj1w0tyhhrfra6swj0aa0ewfhxpuu5wv4uyj0uc0w"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"

sleep 1

#DEPOSIT='{"deposit":{}}'
## shellcheck disable=SC2046
## shellcheck disable=SC2116
#yes 12345678 | injectived tx wasm execute $CONTRACT "$DEPOSIT" --from=$(echo $INJ_ADDRESS) --amount="100peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443

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
