#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{
        Addr,
        Uint128
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        QueryMsg,
        GetUserDepositedUsdResponse
    };

    #[test]
    fn test_get_available_to_redeem() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18
        const BORROW_AMOUNT_ATOM: u128 = 1000 * DECIMAL_FRACTIONAL.u128();

        let user_deposited_usd: GetUserDepositedUsdResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserDepositedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_usd.user_deposited_usd.u128(),
            40300000000000 // 300 ATOM * 10$ + 200 ETH * 2000$ = 403000$
        );

        let available_to_redeem_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_to_redeem_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(available_to_redeem_eth.u128(), 200000000000000000000); // 200 ETH == 400000$
        assert_eq!(available_to_redeem_atom.u128(), 300000000000000000000); // 300 ATOM == 3000$

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ATOM),
            },
            &[],
        )
        .unwrap();
        
        let available_to_redeem_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_to_redeem_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // required_collateral_balance_usd = BORROW_AMOUNT_ATOM * PRICE / 0.8 = 1000 ATOM * 10$ / 0.8 = 12500$
        // user_deposited_usd - required_collateral_balance_usd = 403000$ - 12500$ = 390500$
        assert_eq!(available_to_redeem_eth.u128(), 195250000000000000000); // 195.25 ETH == 390500$
        assert_eq!(available_to_redeem_atom.u128(), 300000000000000000000); // 300 ATOM == 3000$
    }
}
