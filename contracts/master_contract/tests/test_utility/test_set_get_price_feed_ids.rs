#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use pyth_sdk_cw::PriceIdentifier;
    use master_contract::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_pyth_contract() {
        let (mut app, addr) = success_deposit_of_one_token_setup();

        let initial_price_feed_ids = vec![
            (
                "inj".to_string(),
                PriceIdentifier::from_hex(
                    "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                )
                    .unwrap(),
            ),
            (
                "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                PriceIdentifier::from_hex(
                    "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                )
                    .unwrap(),
            ),
        ];

        const PRICE_DECIMALS: u32 = 8;
        const PRICE_ETH: u128 = 2000u128 * 10u128.pow(PRICE_DECIMALS);
        const PRICE_ATOM: u128 = 10u128 * 10u128.pow(PRICE_DECIMALS);

        let price_feed_ids: Vec<(String, PriceIdentifier)> = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPriceFeedIds {},
            )
            .unwrap();

        assert_eq!(initial_price_feed_ids, price_feed_ids);
    }

    #[test]
    fn test_add_price_feed_ids() {
        let (mut app, addr) = success_deposit_of_one_token_setup();

        let initial_price_feed_ids = vec![
            (
                "inj".to_string(),
                PriceIdentifier::from_hex(
                    "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                )
                    .unwrap(),
            ),
            (
                "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                PriceIdentifier::from_hex(
                    "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                )
                    .unwrap(),
            ),
        ];

        const PRICE_DECIMALS: u32 = 8;
        const PRICE_ETH: u128 = 2000u128 * 10u128.pow(PRICE_DECIMALS);
        const PRICE_ATOM: u128 = 10u128 * 10u128.pow(PRICE_DECIMALS);

        let mut price_feed_ids: Vec<(String, PriceIdentifier)> = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPriceFeedIds {},
            )
            .unwrap();

        assert_eq!(initial_price_feed_ids, price_feed_ids);


        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AddPriceFeedIds {
                price_ids: vec![(
                    "ANOTHER_TOKEN_DENOM".to_string(),
                    PriceIdentifier::from_hex(
                        "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
                    )
                        .unwrap(),
                )],
            },
            &[],
        ).unwrap();

        let price_feed_ids_after: Vec<(String, PriceIdentifier)> = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPriceFeedIds {},
            )
            .unwrap();

        assert_ne!(price_feed_ids_after, price_feed_ids);

        assert_eq!(price_feed_ids_after.len(), price_feed_ids.len() + 1);

        // Exactly the same we were adding
        assert_eq!(price_feed_ids_after.first().unwrap(), &(
            "ANOTHER_TOKEN_DENOM".to_string(),
            PriceIdentifier::from_hex(
                "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            )
                .unwrap(),
        ));
    }
}
