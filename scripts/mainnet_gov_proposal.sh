injectived tx wasm submit-proposal wasm-store ../artifacts/lending-aarch64.wasm \
--title "iLend contract upgrade (admin functions for markets management)" \
--summary "iLend is a decentralized finance protocol developed on the Injective network.\n It offers the ability to engage in lending and borrowing activities in a decentralized, transparent, and efficient manner. The protocol leverages Injective Protocol's ability to offer fast, secure, and EVM-compatible DeFi transactions across multiple blockchain ecosystems.\n \n Current upgrade provides new admin functions to add new markets for supply and borrow and liquidation function upgrade that allows any user to be a liquidator.\n Github: https://github.com/i-Lend-org/iLend-smart-contracts\n Built by zpoken team https://zpoken.io/" \
--instantiate-anyof-addresses inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z \
--deposit=10000000000000000000inj \
--gas=30000000 \
--chain-id=injective-1 \
--broadcast-mode=sync \
--node https://sentry.tm.injective.network:443 \
--yes \
--from inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z \
--gas-prices=500000000inj



