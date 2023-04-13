# get the contract address from the response
CODE_ID="648"
injectived query wasm list-contract-by-code $CODE_ID --node=https://k8s.testnet.tm.injective.network:443 --output json


CONTRACT="inj1uvrxlvupxmsyqj752kwasdtd6kpcgpc9a8rke3"

## query arbitrary balance
BALANCE_QUERY='{"get_deposit": {"address": "'$(echo $INJ_ADDRESS)'", "denom": "eth"}}'
injectived query wasm contract-state smart $CONTRACT "$BALANCE_QUERY" --node=https://k8s.testnet.tm.injective.network:443 --output json

