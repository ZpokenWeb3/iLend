#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg, UserDataByToken};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_get_total_reserves_by_token() {
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        let user_data_by_token: Vec<(String, UserDataByToken)> = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBalances {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        println!("{:?}", user_data_by_token);
    }
}
