#!/bin/bash

source ../.env

readonly CONTRACT="inj1qunaqndmy45x6sa0uht09ve0n4a8cuvmsmqfm0"
readonly INJ_ADDRESS="inj19ae4ukagwrlprva55q9skskunv5ve7sr6myx7z"
readonly DENOM_USDT="peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5"

# shellcheck disable=SC2046
# shellcheck disable=SC2116

get_query() {
  injectived query wasm contract-state smart $CONTRACT \"$1\" --node=https://k8s.testnet.tm.injective.network:443 --output json
  sleep 3
  echo
}

execute_query() {
  yes 12345678 | injectived tx wasm execute $CONTRACT \"$1\" --from=$INJ_ADDRESS --chain-id=\"injective-888\" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
  sleep 3
  echo
}

execute_query_with_amount() {
  yes 12345678 | injectived tx wasm execute $CONTRACT \"$1\" --from=$INJ_ADDRESS --amount=\"$2\" --chain-id=\"injective-888\" --yes --gas-prices=500000000inj --gas=20000000 --node=https://k8s.testnet.tm.injective.network:443
  sleep 3
  echo
}

echo deposit USDT
EXECUTE_MSG='{\"deposit\":{}}'
AMOUNT=10000$DENOM_USDT
execute_query_with_amount "$EXECUTE_MSG" "$AMOUNT"

echo get_price for USDT
QUERY='{\"get_price\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_deposit for USDT
QUERY='{\"get_deposit\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_user_deposited_usd
QUERY='{\"get_user_deposited_usd\": {\"address\": \"'$(echo $INJ_ADDRESS)'\"}}'
get_query "$QUERY"

echo get_user_collateral_usd
QUERY='{\"get_user_collateral_usd\": {\"address\": \"'$(echo $INJ_ADDRESS)'\"}}'
get_query "$QUERY"

echo user_deposit_as_collateral for USDT
QUERY='{\"user_deposit_as_collateral\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
result=$(sudo docker exec -it injective-core-v1.10.1 sh -c "injectived query wasm contract-state smart $CONTRACT \"$QUERY\" --node=https://k8s.testnet.tm.injective.network:443 --output json")
sleep 3
echo

if [[ $(echo "$result" | jq -r '.data') == "false" ]]; then
  echo toggle_collateral_setting for USDT
  EXECUTE_MSG='{\"toggle_collateral_setting\":{\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
  execute_query "$EXECUTE_MSG"

  echo user_deposit_as_collateral for USDT
  QUERY='{\"user_deposit_as_collateral\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
  get_query "$QUERY"
fi

echo get_available_to_borrow for USDT
QUERY='{\"get_available_to_borrow\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo borrow USDT 
EXECUTE_MSG='{\"borrow\":{\"denom\":\"'$(echo $DENOM_USDT)'\",\"amount\":\"2000\"}}'
execute_query "$EXECUTE_MSG"

echo get_user_borrow_amount_with_interest for USDT
QUERY='{\"get_user_borrow_amount_with_interest\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_utilization_rate_by_token for USDT
QUERY='{\"get_utilization_rate_by_token\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_user_borrowed_usd
QUERY='{\"get_user_borrowed_usd\": {\"address\": \"'$(echo $INJ_ADDRESS)'\"}}'
get_query "$QUERY"

echo get_interest_rate for USDT
QUERY='{\"get_interest_rate\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_liquidity_rate for USDT
QUERY='{\"get_liquidity_rate\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_available_liquidity_by_token for USDT
QUERY='{\"get_available_liquidity_by_token\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_total_borrowed_by_token for USDT
QUERY='{\"get_total_borrowed_by_token\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_total_deposited_by_token for USDT
QUERY='{\"get_total_deposited_by_token\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_total_reserves_by_token for USDT
QUERY='{\"get_total_reserves_by_token\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_user_utilization_rate
QUERY='{\"get_user_utilization_rate\": {\"address\": \"'$(echo $INJ_ADDRESS)'\"}}'
get_query "$QUERY"

echo get_utilization_rate_by_token for USDT
QUERY='{\"get_utilization_rate_by_token\": {\"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_available_to_redeem for USDT
QUERY='{\"get_available_to_redeem\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo redeem USDT 
EXECUTE_MSG='{\"redeem\":{\"denom\":\"'$(echo $DENOM_USDT)'\",\"amount\":\"3000\"}}'
execute_query "$EXECUTE_MSG"

echo repay USDT
EXECUTE_MSG='{\"repay\":{}}'
AMOUNT=1000$DENOM_USDT
execute_query_with_amount "$EXECUTE_MSG" "$AMOUNT"

echo repay USDT
EXECUTE_MSG='{\"repay\":{}}'
AMOUNT=10000$DENOM_USDT
execute_query_with_amount "$EXECUTE_MSG" "$AMOUNT"

echo get_user_borrow_amount_with_interest for USDT
QUERY='{\"get_user_borrow_amount_with_interest\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo get_available_to_redeem for USDT
QUERY='{\"get_available_to_redeem\": {\"address\": \"'$(echo $INJ_ADDRESS)'\", \"denom\": \"'$(echo $DENOM_USDT)'\"}}'
get_query "$QUERY"

echo redeem USDT 
EXECUTE_MSG='{\"redeem\":{\"denom\":\"'$(echo $DENOM_USDT)'\",\"amount\":\"10000\"}}'
execute_query "$EXECUTE_MSG"
