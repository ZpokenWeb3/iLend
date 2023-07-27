#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_utilization_rate_by_token() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 10 * 10u128.pow(TOKENS_DECIMALS); // 10 ETH
        const BORROW_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ATOM

        let utilization_rate_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUtilizationRateByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let utilization_rate_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUtilizationRateByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // utilization rate is zero, since no one has borrowed anything yet
        assert_eq!(utilization_rate_eth.u128(), 0);
        assert_eq!(utilization_rate_atom.u128(), 0);

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

        let utilization_rate_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUtilizationRateByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let utilization_rate_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUtilizationRateByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(utilization_rate_eth.u128(), 83333); // 0.83333% == 188% * 10/(1000 + 200)
        assert_eq!(utilization_rate_atom.u128(), 1538461); // 15.38461% == 188% * 200/(1000 + 300)
    }
}
