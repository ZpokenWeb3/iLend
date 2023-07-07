# get the contract address from explorer tx of instantiating
CONTRACT="inj1znmfkan4qpttv2rumzvwhcpfjju24lpvdz5a9y"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"

SET_PRICE_APE='{"update_price": {"denom": "inj", "price": "1"}}'
yes 12345678 | injectived tx wasm execute $CONTRACT "$SET_PRICE_APE" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
#
#sleep 2

GET_PRICE='{"get_price": {"denom": "peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5"}}'
injectived query wasm contract-state smart $CONTRACT "$GET_PRICE" --node=https://k8s.testnet.tm.injective.network:443 --output json

sleep 2

GET_PRICE='{"get_price": {"denom": "inj"}}'
injectived query wasm contract-state smart $CONTRACT "$GET_PRICE" --node=https://k8s.testnet.tm.injective.network:443 --output json

sleep 2

GET_PRICE='{"get_price": {"denom": "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7"}}'
injectived query wasm contract-state smart $CONTRACT "$GET_PRICE" --node=https://k8s.testnet.tm.injective.network:443 --output json

sleep 2
