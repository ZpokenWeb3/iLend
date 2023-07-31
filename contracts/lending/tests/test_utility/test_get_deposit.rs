#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_one_token_setup;
    use lending::msg::{GetBalanceResponse, QueryMsg};

    #[test]
    fn test_get_deposit() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (app, lending_contract_addr, _collateral_contract_addr) =
            success_deposit_of_one_token_setup();

        let user_deposit_amount_eth: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_deposit_amount_atom: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposit_amount_atom.balance.u128(), 0); // 0
        assert_eq!(
            user_deposit_amount_eth.balance.u128(),
            500000000000000000000
        ); // 500
    }
}
