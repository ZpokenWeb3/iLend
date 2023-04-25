# address of user on testnet and received some faucet tokens
readonly INJ_ADDRESS="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd"


yes 12345678 | injectived tx wasm store ../artifacts/master_contract-aarch64.wasm \
--from=$INJ_ADDRESS \
--chain-id="injective-888" \
--yes --gas-prices=500000000inj --gas=20000000 \
--node=https://k8s.testnet.tm.injective.network:443