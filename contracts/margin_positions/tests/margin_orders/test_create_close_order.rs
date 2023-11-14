#[cfg(test)]
mod tests {
    use crate::utils::{success_collateral_margin_setup_with_deposit, success_setup_three_contracts};
    use cosmwasm_std::{Addr, coins, Uint128};
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
                leverage: 100,
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
                leverage: 100,
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
                leverage: 100,
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
                leverage: 100,
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
                leverage: 100,
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
                leverage: 100,
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
                leverage: 100,
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
                leverage: 100,
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

    #[test]
    fn test_borrowed_respective_amount() {
        const TOKENS_DECIMALS: u32 = 18;
        const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);

        let (mut app, _lending_contract_addr, margin_positions_addr, collateral_contract_addr)
            = success_setup_three_contracts();


        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::Deposit {},
            &coins(FIRST_DEPOSIT_AMOUNT_ETH, "eth"),
        )
            .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetDeposit {
                    user: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), FIRST_DEPOSIT_AMOUNT_ETH);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT_ETH
        );

        assert_eq!(
            app.wrap()
                .query_balance(collateral_contract_addr.clone(), "eth")
                .unwrap()
                .amount
                .u128(),
            RESERVE_AMOUNT + FIRST_DEPOSIT_AMOUNT_ETH
        );

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


        assert_eq!(
            app.wrap()
                .query_balance(margin_positions_addr.clone(), "eth")
                .unwrap()
                .amount
                .u128(),
            FIRST_DEPOSIT_AMOUNT_ETH
        );

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


        assert_eq!(
            app.wrap()
                .query_balance(margin_positions_addr.clone(), "eth")
                .unwrap()
                .amount
                .u128(),
            FIRST_DEPOSIT_AMOUNT_ETH * 2
        );
    }

}
