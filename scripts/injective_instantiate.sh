# step 1 query CODE_ID from deployed contract via command
#readonly DEPLOYMENT_TX_HASH="DA58BC975F2B8F2CF307FD463571D0CA65E0EAD74A45F1B5A712FD3D953D8B1B"
#injectived query tx $DEPLOYMENT_TX_HASH --node=https://k8s.testnet.tm.injective.network:443


readonly CODE_ID="631"


# instantiate contract
yes 12345678 | injectived tx wasm instantiate $CODE_ID '{}' --label="Injective Test" --from="inj1lsuerzge89tyd4p2pj8wrj903v5ja5emmugntd" --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --no-admin --node=https://k8s.testnet.tm.injective.network:443


