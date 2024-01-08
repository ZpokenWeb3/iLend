injectived tx wasm submit-proposal wasm-store ../artifacts/lending-aarch64.wasm \
--title "Upload the iLend Protocol contract" \
--summary "iLend Protocol: \n iLend is a decentralized finance protocol developed on the Injective network. It offers the ability to engage in lending and borrowing activities in a decentralized, transparent, and efficient manner. The protocol leverages Injective Protocol's ability to offer fast, secure, and EVM-compatible DeFi transactions across multiple blockchain ecosystems \n github: https://github.com/i-Lend-org/iLend-smart-contracts \n build by zpoken team https://zpoken.io/" \
--instantiate-anyof-addresses inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z \
--gas=30000000 \
--chain-id=injective-1 \
--broadcast-mode=sync \
--node https://sentry.tm.injective.network:443 \
--yes \
--from inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z \
--gas-prices=500000000inj



