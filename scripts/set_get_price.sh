# get the contract address from explorer tx of instantiating
CONTRACT="inj1znmfkan4qpttv2rumzvwhcpfjju24lpvdz5a9y"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"

UPDATE_PRICE='{"update_price": {"denom": "inj", "price": "1"}}'
yes 12345678 | injectived tx wasm execute $CONTRACT "$UPDATE_PRICE" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
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

injectived tx wasm execute inj1xjkfkfgjg60gh3duf5hyk3vfsluyurjljznwgu '{"add_price_feed_ids": [["factory/inj14ejqjyq8um4p3xfqj74yld5waqljf88f9eneuk/inj18luqttqyckgpddndh8hvaq25d5nfwjc78m56lc","7a5bc1d2b56ad029048cd63964b3ad2776eadf812edc1a43a31406cb54bff592"]]}' --from=inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z --chain-id="injective-1" --yes --gas-prices=500000000inj --gas=20000000 --node=https://sentry.tm.injective.network:443 --output json


"price_ids":[["inj","7a5bc1d2b56ad029048cd63964b3ad2776eadf812edc1a43a31406cb54bff592"]]