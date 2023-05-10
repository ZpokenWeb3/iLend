#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
        use cosmwasm_std::{
            Addr,
//             Uint128
        };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        QueryMsg,
    };

    #[test]
    fn test_user_deposit_as_collateral() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_of_one_token_setup();

        let user_eth_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_atom_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_eth_deposit_as_collateral, false);
        assert_eq!(user_atom_deposit_as_collateral, false);

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
        )
        .unwrap();

        let user_eth_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_atom_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_eth_deposit_as_collateral, true);
        assert_eq!(user_atom_deposit_as_collateral, false);
    }
}
