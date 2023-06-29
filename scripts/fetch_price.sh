# get the contract address from explorer tx of instantiating
CONTRACT="inj1gmk2w8mqcwwm4rwlqthksuyqvk3vnwu5vj05pt"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"

FETCH_PRICE_INJ='{"fetch_price": {"denom": "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7"}}'
injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_INJ" --node=https://k8s.testnet.tm.injective.network:443 --output json
