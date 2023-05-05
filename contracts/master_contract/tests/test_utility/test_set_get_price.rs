#[cfg(test)]
mod tests {
//     use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
//     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{
        Addr,
//         Uint128
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, GetPriceResponse, QueryMsg};

    #[test]
    fn test_set_get_price() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_of_one_token_setup();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetPrice {
                denom: "eth".to_string(),
                price: 2000,
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetPrice {
                denom: "atom".to_string(),
                price: 10,
            },
            &[],
        )
        .unwrap();

        let get_price_eth: GetPriceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPrice {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let get_price_atom: GetPriceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPrice {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_price_atom.price, 10);

        assert_eq!(get_price_eth.price, 2000);
    }
}
