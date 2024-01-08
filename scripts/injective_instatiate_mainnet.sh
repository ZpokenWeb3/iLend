# step 1 query CODE_ID from deployed contract via command
#readonly DEPLOYMENT_TX_HASH="0AD47AC425C0133910A3654368DDDF2AEA276C35E068D7D44581AC37ECCD809C"
#injectived query tx $DEPLOYMENT_TX_HASH --node=https://k8s.testnet.tm.injective.network:443

readonly CODE_ID="6040"
readonly INJ_ADDRESS="inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z"

# shellcheck disable=SC2089
#  supported tokens arguments denom, name, symbol, decimals
INIT='{
    "price_updater_addr": "-----",
    "reserve_configuration":
      [
        ["inj", "8500000", "9000000"],
        ["peggy0xdAC17F958D2ee523a2206206994597C13D831ec7" ,"7500000", "8000000"],
        ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9" ,"", ""],
      ],
    "is_testing": false,
    "liquidator": "------",
    "admin": "------",
    "price_ids":
      [
        ["inj", "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3"],
        ["peggy0xdAC17F958D2ee523a2206206994597C13D831ec7", "1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588"],
        ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9", "b00b60f88b03a6a625a8d1c048c3f66653edf217439983d037e7222c4e612819"],
      ],
    "pyth_contract_addr": "inj12j43nf2f0qumnt2zrrmpvnsqgzndxefujlvr08", // TODO https://docs.pyth.network/documentation/pythnet-price-feeds/cosmwasm
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
        ["peggy0xdAC17F958D2ee523a2206206994597C13D831ec7", "5000000000000000000", "20000000000000000000", "100000000000000000000", "8000000"],
        ["inj", "5000000000000000000", "40000000000000000000", "70000000000000000000", "8000000"],
        ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9", "5000000000000000000", "50000000000000000000", "60000000000000000000", "8000000"]
       ]
      }'

# shellcheck disable=SC2046
yes 12345678 | injectived tx wasm instantiate $CODE_ID $INIT --label="iLend Contract" --from=$(echo $INJ_ADDRESS) --chain-id="injective-1" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://sentry.tm.injective.network:443
