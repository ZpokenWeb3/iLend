# get the contract address from the response
# injectived query wasm list-contract-by-code 631 --node=https://k8s.testnet.tm.injective.network:443 --output json

# transferring tokens

  readonly CONTRACT="inj1ayxl9u84k04ccxnde5yz94xynw2h7qsdw7fnxq"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"
# shellcheck disable=SC2016
readonly TRANSFER='{"transfer":{"recipient":"'$CONTRACT'","amount":"420"}}'

yes 12345678 | injectived tx wasm execute "$CONTRACT" "$TRANSFER" --from=$INJ_ADDRESS --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
