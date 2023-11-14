#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use lending::msg::QueryMsg;

    #[test]
    fn test_user_deposit_as_collateral() {
        let (app, lending_contract_addr, _collateral_contract_addr) =
            success_deposit_of_diff_token_with_prices();

        let user_eth_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_atom_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_eth_deposit_as_collateral, false);
        assert_eq!(user_atom_deposit_as_collateral, false);
    }
}
