# step 1 query CODE_ID from deployed contract via command
#readonly DEPLOYMENT_TX_HASH="0AD47AC425C0133910A3654368DDDF2AEA276C35E068D7D44581AC37ECCD809C"
#injectived query tx $DEPLOYMENT_TX_HASH --node=https://k8s.testnet.tm.injective.network:443

readonly CODE_ID="2013"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"

# shellcheck disable=SC2089
#  supported tokens arguments denom, name, symbol, decimals
INIT='{
    "price_updater_contract_addr": "inj1w8surqt52asamxlww9kc73hy8x3clh55s5k44q",
    "reserve_configuration":
      [
        ["inj", "8500000", "9000000"],
        ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7","7500000", "8000000"],
        ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" ,"7500000", "8000000"]
      ],
    "is_testing": false,
    "liquidator": "inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd",
    "admin": "inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd",
    "price_ids":
      [
        ["inj", "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3"],
        ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7", "cb1743d0e3e3eace7e84b8230dc082829813e3ab04e91b503c08e9a441c0ea8b"],
        ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722"]
      ],
    "pyth_contract_addr": "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez",
    "supported_tokens":
      [
        ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "Tether", "USDT", "6"],
        ["inj", "Injective", "INJ", "18"],
        ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7", "Ape Coin", "APE", "18"]
      ],
    "tokens_interest_rate_model_params":
      [
        ["peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5", "5000000000000000000", "20000000000000000000", "100000000000000000000", "8000000"],
        ["inj", "5000000000000000000", "40000000000000000000", "70000000000000000000", "8000000"],
        ["peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7", "5000000000000000000", "50000000000000000000", "60000000000000000000", "8000000"]
       ]
      }'

# shellcheck disable=SC2046
yes 12345678 | injectived tx wasm instantiate $CODE_ID $INIT --label="iLend Contract" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://k8s.testnet.tm.injective.network:443
