#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::Uint128;
    //     use cw_multi_test::Executor;
    use lending::msg::QueryMsg;

    #[test]
    fn test_get_interest_rate() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (app, lending_contract_addr, _collateral_contract_addr) =
            success_deposit_of_one_token_setup();

        let get_interest_rate_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetInterestRate {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let get_interest_rate_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetInterestRate {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_interest_rate_atom.u128(), 5000000000000000000); // 5%

        assert_eq!(get_interest_rate_eth.u128(), 5000000000000000000); // 5%
    }
}
