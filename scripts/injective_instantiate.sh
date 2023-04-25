# step 1 query CODE_ID from deployed contract via command
#readonly DEPLOYMENT_TX_HASH="7215C4626DB10D9822B18BBBE76D5D26A705FF739C7A9460F122AB4649BF2674"
#injectived query tx $DEPLOYMENT_TX_HASH --node=https://k8s.testnet.tm.injective.network:443

CODE_ID="811"
INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"


# shellcheck disable=SC2089
#  supported tokens arguments denom, name, symbol, decimals
INIT='{"admin":"inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd","supported_tokens": [["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "Tether", "USDT", "6"], ["inj", "Injective", "INJ", "18"], ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7", "Ape Coin", "APE", "18"]]}'


# shellcheck disable=SC2046
yes 12345678 | injectived tx wasm instantiate $CODE_ID $INIT --label="iLend Contract" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://k8s.testnet.tm.injective.network:443