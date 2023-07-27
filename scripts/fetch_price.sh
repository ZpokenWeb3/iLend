# get the contract address from explorer tx of instantiating
CONTRACT="inj1ulkyckufg8f0q20nsavcq5shcttq0n8nlc39t4"
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"

FETCH_PRICE_INJ='{"get_all_users_with_borrows": {}}'
injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_INJ" --node=https://k8s.testnet.tm.injective.network:443 --output json

FETCH_PRICE_INJ='{"get_user_utilization_rate": {"address": "inj1f0kp66zuyr6j7rc43m02h5y7wu8tfnra4hsrwn"}}'
injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_INJ" --node=https://k8s.testnet.tm.injective.network:443 --output json

FETCH_PRICE_INJ='{"get_user_liquidation_threshold": {"address": "inj1f0kp66zuyr6j7rc43m02h5y7wu8tfnra4hsrwn"}}'
injectived query wasm contract-state smart $CONTRACT "$FETCH_PRICE_INJ" --node=https://k8s.testnet.tm.injective.network:443 --output json
