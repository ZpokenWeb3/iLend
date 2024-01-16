readonly CODE_ID="6319"
readonly INJ_ADDRESS="inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z"

# shellcheck disable=SC2046
# shellcheck disable=SC2090
yes 12345678 | injectived tx wasm instantiate $CODE_ID '{
                                                           "is_testing": false,
                                                           "admin": "inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z",
                                                           "price_updater_addr": "inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z",
                                                           "pyth_contract_addr": "inj18rlflp3735h25jmjx97d22c72sxk260amdjxlu",
                                                           "price_ids": [
                                                             ["inj", "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3"],
                                                             ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722"]
                                                           ],
                                                           "reserve_configuration": [
                                                             ["inj", "8500000", "9000000"],
                                                             ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "7500000", "8000000"]
                                                           ],
                                                           "supported_tokens": [
                                                             ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "Tether", "USDT", "6"],
                                                             ["inj", "Injective", "INJ", "18"]
                                                           ],
                                                           "tokens_interest_rate_model_params": [
                                                             ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "5000000000000000000", "20000000000000000000", "100000000000000000000", "8000000"],
                                                             ["inj", "5000000000000000000", "40000000000000000000", "70000000000000000000", "8000000"]
                                                           ]
                                                       }' --label="iLend Contract" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://k8s.testnet.tm.injective.network:443