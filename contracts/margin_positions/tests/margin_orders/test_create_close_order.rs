#[cfg(test)]
mod tests {
    use crate::utils::success_collateral_margin_setup_with_deposit;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;

    use margin_positions::msg::{
        ExecuteMsg as ExecuteMsgMarginPositions, OrderResponse, QueryMsg as QueryMsgMarginPositions,
    };
    use margin_positions::utils::{OrderStatus, OrderType};

    #[test]
    fn test_create_order() {
        const TOKENS_DECIMALS: u32 = 18;
        const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
        let (mut app, margin_positions_addr, _collateral_contract_addr) =
            success_collateral_margin_setup_with_deposit();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::CreateOrder {
                order_type: OrderType::Short,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                sell_token_denom: "eth".to_string(),
                leverage: 125,
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::CreateOrder {
                order_type: OrderType::Long,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                sell_token_denom: "eth".to_string(),
                leverage: 200,
            },
            &[],
        )
        .unwrap();

        let user_orders: Vec<OrderResponse> = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetOrdersByUser {
                    user: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_orders.len(), 2);

        assert_eq!(
            *user_orders.first().unwrap(),
            OrderResponse {
                order_status: OrderStatus::Pending,
                order_type: OrderType::Short,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                leverage: 125,
                sell_token_denom: "eth".to_string(),
            },
            "Have to be exactly the same order we have created"
        )
    }

    #[test]
    fn test_get_order_by_id() {
        const TOKENS_DECIMALS: u32 = 18;
        const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
        let (mut app, margin_positions_addr, _collateral_contract_addr) =
            success_collateral_margin_setup_with_deposit();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::CreateOrder {
                order_type: OrderType::Short,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                sell_token_denom: "eth".to_string(),
                leverage: 125,
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::CreateOrder {
                order_type: OrderType::Long,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                sell_token_denom: "eth".to_string(),
                leverage: 200,
            },
            &[],
        )
        .unwrap();

        let user_orders: Vec<OrderResponse> = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetOrdersByUser {
                    user: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_orders.len(), 2);

        assert_eq!(
            *user_orders.first().unwrap(),
            OrderResponse {
                order_status: OrderStatus::Pending,
                order_type: OrderType::Short,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                leverage: 125,
                sell_token_denom: "eth".to_string(),
            },
            "Have to be exactly the same order we have created"
        );

        let order_number_2: OrderResponse = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetOrderById { order_id: 2 },
            )
            .unwrap();

        assert_eq!(
            order_number_2,
            OrderResponse {
                order_status: OrderStatus::Pending,
                order_type: OrderType::Long,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                sell_token_denom: "eth".to_string(),
                leverage: 200,
            },
            "Have to be exactly the same order we have created"
        );
    }

    #[test]
    fn test_close_order() {
        const TOKENS_DECIMALS: u32 = 18;
        const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
        let (mut app, margin_positions_addr, _collateral_contract_addr) =
            success_collateral_margin_setup_with_deposit();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::CreateOrder {
                order_type: OrderType::Long,
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT_ETH / 2),
                sell_token_denom: "eth".to_string(),
                leverage: 200,
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::CancelOrder { order_id: 1 },
            &[],
        )
        .unwrap();

        let user_orders: Vec<OrderResponse> = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetOrdersByUser {
                    user: "user".to_string(),
                },
            )
            .unwrap();

        dbg!(user_orders.first());
    }
}
