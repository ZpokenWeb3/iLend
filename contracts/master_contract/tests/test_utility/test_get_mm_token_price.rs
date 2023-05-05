#[cfg(test)]
mod tests {
//     use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
//     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{
//         Addr,
        Uint128
    };
//     use cw_multi_test::Executor;
    use master_contract::msg::{
//         ExecuteMsg,
        QueryMsg
    };

    #[test]
    fn test_get_mm_token_price() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (app, addr) = success_deposit_of_one_token_setup();

        let get_mm_token_price_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetMmTokenPrice {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let get_mm_token_price_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetMmTokenPrice {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_mm_token_price_atom.u128(), 1000000000000000000); // 1:1
        assert_eq!(get_mm_token_price_eth.u128(), 1000000000000000000); // 1:1
    }
}
