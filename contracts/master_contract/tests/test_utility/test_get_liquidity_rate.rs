#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_liquidity_rate() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18
        const BORROW_SECOND_TOKEN_FIRST_PART: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_SECOND_TOKEN_FIRST_PART),
            },
            &[],
        )
        .unwrap();

        let get_liquidity_rate_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetLiquidityRate {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let get_liquidity_rate_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetLiquidityRate {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_liquidity_rate_atom.u128(), 1499999999550000000);
        assert_eq!(get_liquidity_rate_eth.u128(), 0);
    }
}
