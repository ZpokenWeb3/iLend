#[cfg(test)]
mod tests {
    use crate::utils::success_borrow_setup;
    use cosmwasm_std::{coin, Addr};
    use cw_multi_test::Executor;
    use lending::msg::ExecuteMsg;

    #[test]
    #[should_panic(expected = "CoinNotFound")]
    fn test_fail_repay_if_funds_not_transferred() {
        // user borrowed 50 ETH
        let (mut app, addr) = success_borrow_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &[],
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Sent more than one denomination")]
    fn test_fail_repay_if_more_than_one_asset_is_transferred() {
        const TOKENS_DECIMALS: u32 = 18;
        const REPAY_AMOUNT_ETH: u128 = 30 * 10u128.pow(TOKENS_DECIMALS); // 30 ETH
        const REPAY_AMOUNT_ATOM: u128 = 3000 * 10u128.pow(TOKENS_DECIMALS); // 3000 ATOM

        // user borrowed 50 ETH
        let (mut app, addr) = success_borrow_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &[
                coin(REPAY_AMOUNT_ETH, "eth"),
                coin(REPAY_AMOUNT_ATOM, "atom"),
            ],
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Cannot transfer empty coins amount")]
    fn test_fail_repay_if_repay_amount_is_zero() {
        // user borrowed 50 ETH
        let (mut app, addr) = success_borrow_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &[coin(0, "eth")],
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Token Not Supported")]
    fn test_fail_repay_if_token_is_not_supported() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_UNSUPPORTED_TOKEN: u128 = 10 * 10u128.pow(TOKENS_DECIMALS);

        // user borrowed 50 ETH
        let (mut app, addr) = success_borrow_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &[coin(BORROW_AMOUNT_UNSUPPORTED_TOKEN, "usdt")],
        )
        .unwrap();
    }
}
