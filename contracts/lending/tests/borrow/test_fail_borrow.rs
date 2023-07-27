#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::ExecuteMsg;

    #[test]
    #[should_panic(expected = "There is no such supported token yet")]
    fn test_fail_borrow_if_token_is_not_supported() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_UNSUPPORTED_TOKEN: u128 = 10 * 10u128.pow(TOKENS_DECIMALS);

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "usdt".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_UNSUPPORTED_TOKEN),
            },
            &[],
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "The amount to be borrowed is not available")]
    fn test_fail_borrow_if_amount_to_be_borrowed_is_not_available() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 300 * 10u128.pow(TOKENS_DECIMALS); // 300 ETH

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ETH), // 300 ETH
            },
            &[],
        )
        .unwrap();
    }
}
