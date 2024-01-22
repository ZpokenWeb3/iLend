readonly INJ_ADDRESS="inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z"
readonly CODE_ID="319"

# shellcheck disable=SC2089
#  supported tokens arguments denom, name, symbol, decimals
INIT='{
    "price_updater_addr": "inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z",
    "reserve_configuration":
      [
        ["inj", "8050000", "8300000"],
        ["peggy0xdAC17F958D2ee523a2206206994597C13D831ec7" ,"7400000", "7600000"],
        ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9" ,"5300000", "6800000"],

        ["peggy0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2" ,"7500000", "8000000"],
        ["ibc/F51BB221BAA275F2EBF654F70B005627D7E713AFFD6D86AFD1E43CAA886149F4" ,"5300000", "6800000"],
      ],
    "is_testing": false,
    "admin": "inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z",
    "price_ids":
      [
        ["inj", "7a5bc1d2b56ad029048cd63964b3ad2776eadf812edc1a43a31406cb54bff592"],
        ["peggy0xdAC17F958D2ee523a2206206994597C13D831ec7", "2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b"],
        ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9", "b00b60f88b03a6a625a8d1c048c3f66653edf217439983d037e7222c4e612819"],

        ["peggy0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace"],
        ["ibc/F51BB221BAA275F2EBF654F70B005627D7E713AFFD6D86AFD1E43CAA886149F4", "09f7c1d7dfbb7df2b8fe3d3d87ee94a2259d212da4f30c1f0540d066dfa44723"],
      ],
    "pyth_contract_addr": "inj12j43nf2f0qumnt2zrrmpvnsqgzndxefujlvr08", // TODO check https://docs.pyth.network/documentation/pythnet-price-feeds/cosmwasm
    "supported_tokens":
      [
        ["inj", "Injective", "INJ", "18"],
        ["peggy0xdAC17F958D2ee523a2206206994597C13D831ec7", "Tether", "USDT", "6"],
        ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9", "Cosmos", "ATOM", "6"]

#        ["peggy0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", "Wrapped Ethereum", "WETH", "18"],
#        ["ibc/F51BB221BAA275F2EBF654F70B005627D7E713AFFD6D86AFD1E43CAA886149F4", "Celestia", "TIA", "6"]
      ],
    "tokens_interest_rate_model_params":
      [
        ["inj", "5000000000000000000", "40000000000000000000", "70000000000000000000", "8000000"],
        ["peggy0xdAC17F958D2ee523a2206206994597C13D831ec7", "3000000000000000000", "60000000000000000000", "100000000000000000000", "7700000"],
        ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9", "5000000000000000000", "70000000000000000000", "60000000000000000000", "8300000"]

        ["peggy0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", "4000000000000000000", "60000000000000000000", "60000000000000000000", "7000000"]
        ["ibc/F51BB221BAA275F2EBF654F70B005627D7E713AFFD6D86AFD1E43CAA886149F4", "40 00000000000000000", "70000000000000000000", "60000000000000000000", "8000000"]
       ]
      }'

# shellcheck disable=SC2046
yes 12345678 | injectived tx wasm instantiate $CODE_ID $INIT --label="iLend Contract" --from=$(echo $INJ_ADDRESS) --chain-id="injective-1" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://sentry.tm.injective.network:443
