# address of user on testnet and received some faucet tokens
readonly INJ_ADDRESS="inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z"


yes 12345678 | injectived tx wasm store ../artifacts/lending-aarch64.wasm \
--from=$INJ_ADDRESS \
--chain-id="injective-888" \
--yes --gas-prices=500000000inj --gas=20000000 \
--node=https://k8s.testnet.tm.injective.network:443