# step 1 query CODE_ID from deployed contract via command
#readonly DEPLOYMENT_TX_HASH="22821D44594BF1E21435A323C1B45686BC61615DE58004E25B69B9ACB259ABEC"
#injectived query tx $DEPLOYMENT_TX_HASH --node=https://k8s.testnet.tm.injective.network:443

CODE_ID="648"
INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"



# shellcheck disable=SC2089
INIT='{"admin":"inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd","supported_tokens":[["eth", "ieth"]]}'


# shellcheck disable=SC2046
yes 12345678 | injectived tx wasm instantiate $CODE_ID $INIT --label="Money-market Contract" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://k8s.testnet.tm.injective.network:443