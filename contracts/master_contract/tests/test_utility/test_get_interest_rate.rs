#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
    use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, GetInterestRateResponse, QueryMsg};

    #[test]
    fn test_get_interest_rate() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_of_one_token_setup();

        let get_interest_rate_eth: GetInterestRateResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetInterestRate {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let get_interest_rate_atom: GetInterestRateResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetInterestRate {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_interest_rate_atom.interest_rate, 5000000000000000000); // 5%

        assert_eq!(get_interest_rate_eth.interest_rate, 5000000000000000000); // 5%
    }
}
