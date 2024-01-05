#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, BlockInfo, Timestamp, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_get_user_deposited_usd() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 50 * 10u128.pow(TOKENS_DECIMALS); // 50 ETH
        const BORROW_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ATOM

        const YEAR_IN_SECONDS: u64 = 31536000;

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        let user_deposited_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserDepositedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_usd.u128(), 40300000000000); // 200 ETH * 2000 + 300 ATOM * 10 = 403_000$

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now),
            chain_id: "custom_chain_id".to_string(),
        });

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

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now + YEAR_IN_SECONDS),
            chain_id: "custom_chain_id".to_string(),
        });

        let user_deposited_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserDepositedUsd {
                    address: "user".to_string(),
                },
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

        assert!(get_liquidity_rate_atom.u128() > 763300000000000000); // ~0.7633%
        assert!(get_liquidity_rate_eth.u128() > 207900000000000000); // ~0.2079%
        assert!(user_deposited_usd.u128() > 40385400000000); // ~403854$ (~0,212% deposite APY)
    }
}
