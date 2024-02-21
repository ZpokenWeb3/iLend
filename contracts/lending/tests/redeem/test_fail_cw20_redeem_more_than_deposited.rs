#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::success_native_and_cw20_setup;
    use cosmwasm_std::{to_json_binary, Addr, Uint128};
    use cw20::{BalanceResponse, Cw20QueryMsg, Cw20ReceiveMsg};
    use cw20_base::msg::{
        ExecuteMsg as ExecuteMsgCW20, InstantiateMsg as InstantiateMsgCW20,
        QueryMsg as QueryMsgCW20,
    };
    use cw_multi_test::Executor;
    use lending::msg::{
        Cw20HookMsg, ExecuteMsg, GetBalanceResponse, GetSupportedTokensResponse, QueryMsg,
        UserDataByToken,
    };

    #[test]
    fn test_success_deposit_cw20() {
        const PERCENT_DECIMALS: u32 = 5;

        const LTV_TIA: u128 = 85 * 10u128.pow(PERCENT_DECIMALS);

        const LIQUIDATION_THRESHOLD_TIA: u128 = 90 * 10u128.pow(PERCENT_DECIMALS);

        const INTEREST_RATE_DECIMALS: u32 = 18;
        const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
        const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
        const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

        const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);

        let (mut app, lending_addr, cw20_token_addr) = success_native_and_cw20_setup();

        let supported_tokens_response_before: GetSupportedTokensResponse = app
            .wrap()
            .query_wasm_smart(lending_addr.clone(), &QueryMsg::GetSupportedTokens {})
            .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            lending_addr.clone(),
            &ExecuteMsg::AddMarkets {
                denom: "ilend-denom".to_string(),
                name: "Ilend Test Tokens".to_string(),
                symbol: "ILEND".to_string(),
                decimals: 6u128,
                cw20_address: Some(cw20_token_addr.to_string()),
                loan_to_value_ratio: LTV_TIA,
                liquidation_threshold: LIQUIDATION_THRESHOLD_TIA,
                min_interest_rate: MIN_INTEREST_RATE,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE,
                rate_growth_factor: RATE_GROWTH_FACTOR,
                optimal_utilisation_ratio: OPTIMAL_UTILISATION_RATIO,
            },
            &[],
        )
            .unwrap();

        let supported_tokens_response_after: GetSupportedTokensResponse = app
            .wrap()
            .query_wasm_smart(lending_addr.clone(), &QueryMsg::GetSupportedTokens {})
            .unwrap();

        assert_eq!(
            supported_tokens_response_before.supported_tokens.len() + 1,
            supported_tokens_response_after.supported_tokens.len(),
            "Has to insert supported token info"
        );

        let cw20_user_balance_before_deposit: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                cw20_token_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: "cw20-user".to_string(),
                },
            )
            .unwrap();

        let lending_balance_before_deposit: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                cw20_token_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: lending_addr.clone().into_string(),
                },
            )
            .unwrap();

        let hook = Cw20HookMsg::Deposit {
            denom: "ilend-denom".to_string(),
        };

        let send_msg = ExecuteMsgCW20::Send {
            contract: lending_addr.clone().to_string(),
            amount: Uint128::from(100000000u128),
            msg: to_json_binary(&hook).unwrap(),
        };

        (app
            .execute_contract(
                Addr::unchecked("cw20-user"),
                cw20_token_addr.clone(),
                &send_msg,
                &[],
            )
            .unwrap());

        let cw20_user_balance_after_deposit: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                cw20_token_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: "cw20-user".to_string(),
                },
            )
            .unwrap();

        let lending_balance_after_deposit: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                cw20_token_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: lending_addr.clone().into_string(),
                },
            )
            .unwrap();

        assert_eq!(
            cw20_user_balance_before_deposit.balance.u128(),
            cw20_user_balance_after_deposit.balance.u128() + 100000000u128
        );
        assert_eq!(
            lending_balance_after_deposit.balance.u128(),
            lending_balance_before_deposit.balance.u128() + 100000000u128
        );

        let user_deposited_balance: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                lending_addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "cw20-user".to_string(),
                    denom: "ilend-denom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance.balance.u128(),
            100000000u128,
            "Should match deposit amount"
        );

        assert!(app.execute_contract(
            Addr::unchecked("cw20-user"),
            lending_addr.clone(),
            &ExecuteMsg::Redeem {
                denom: "ilend-denom".to_string(),
                amount: Uint128::from(10000000000u128),
            },
            &[],
        ).is_err());

        let user_deposited_balance: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                lending_addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "cw20-user".to_string(),
                    denom: "ilend-denom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance.balance.u128(),
            100000000u128,
            "Should be zero after whole redemption"
        );
    }
}
