# get the contract address from explorer tx of instantiating
CONTRACT="inj12ky27ufq6uru9f9ehttkjxhjfq7sprxf89guvg"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"

FETCH_PRICE_INJ='{"get_price": {"denom": "inj"}}'
injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_INJ" --node=https://k8s.testnet.tm.injective.network:443 --output json
