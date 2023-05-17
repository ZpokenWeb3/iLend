# step 1 query CODE_ID from deployed contract via command
#readonly DEPLOYMENT_TX_HASH="482588C73050DCBE48CC11FBEE250A08606A07E3E05350ED373E6B955DED9846"
#injectived query tx $DEPLOYMENT_TX_HASH --node=https://k8s.testnet.tm.injective.network:443

CODE_ID="1209"
INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"


# shellcheck disable=SC2089
#  supported tokens arguments denom, name, symbol, decimals
INIT='{"admin":"inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd","supported_tokens": [["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "Tether", "USDT", "6"], ["inj", "Injective", "INJ", "18"], ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7", "Ape Coin", "APE", "18"]],"reserve_configuration": [["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "8500000", "9000000"], ["inj", "8000000", "8500000"], ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7", "7000000", "7500000"]],"tokens_interest_rate_model_params": [["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "5000000000000000000", "20000000000000000000", "100000000000000000000"], ["inj", "5000000000000000000", "40000000000000000000", "70000000000000000000"], ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7", "5000000000000000000", "50000000000000000000", "60000000000000000000"]]}'


# shellcheck disable=SC2046
yes 12345678 | injectived tx wasm instantiate $CODE_ID $INIT --label="iLend Contract" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://k8s.testnet.tm.injective.network:443
