#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_set_get_price() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_of_one_token_setup();

        const PRICE_DECIMALS: u32 = 8;
        const PRICE_ETH: u128 = 2000u128 * 10u128.pow(PRICE_DECIMALS);
        const PRICE_ATOM: u128 = 10u128 * 10u128.pow(PRICE_DECIMALS);

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::UpdatePrice {
                denom: Some("eth".to_string()),
                price: Some(PRICE_ETH),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::UpdatePrice {
                denom: Some("atom".to_string()),
                price: Some(PRICE_ATOM),
            },
            &[],
        )
        .unwrap();

        let get_price_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPrice {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let get_price_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPrice {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_price_atom.u128(), 1000000000); // 10$
        assert_eq!(get_price_eth.u128(), 200000000000); // 2000$
    }
}
