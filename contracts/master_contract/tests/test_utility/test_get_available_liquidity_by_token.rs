#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_available_liquidity_by_token() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 100 * 10u128.pow(TOKENS_DECIMALS); // 100 ETH
        const BORROW_AMOUNT_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS); // 1000 ATOM
        const DEPOSIT_AMOUNT_ETH: u128 = 30 * 10u128.pow(TOKENS_DECIMALS); // 30 ETH
        const DEPOSIT_AMOUNT_ATOM: u128 = 400 * 10u128.pow(TOKENS_DECIMALS); // 400 ATOM

        let available_liquidity_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_liquidity_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            available_liquidity_by_token_eth.u128(),
            1200000000000000000000
        ); // 1000 ETH + 200 ETH = 1200 ETH
        assert_eq!(
            available_liquidity_by_token_atom.u128(),
            1300000000000000000000
        ); // 1000 ATOM + 300 ATOM = 1300 ATOM

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ETH),
            },
            &[],
        )
        .unwrap();

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

        let available_liquidity_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_liquidity_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            available_liquidity_by_token_eth.u128(),
            1100000000000000000000
        ); // 1200 ETH - 100 ETH = 1100 ETH
        assert_eq!(
            available_liquidity_by_token_atom.u128(),
            300000000000000000000
        ); // 1300 ATOM - 1000 ATOM = 300 ATOM

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ETH, "eth"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
        )
        .unwrap();

        let available_liquidity_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_liquidity_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            available_liquidity_by_token_eth.u128(),
            1130000000000000000000
        ); // 1100 ETH + 30 ETH = 1130 ETH
        assert_eq!(
            available_liquidity_by_token_atom.u128(),
            700000000000000000000
        ); // 300 ATOM + 400 ATOM = 700 ATOM
    }
}
