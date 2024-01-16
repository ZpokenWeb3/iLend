#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_one_token_setup;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_update_admin() {
        let (mut app, addr) = success_deposit_of_one_token_setup();

        let old_admin: String = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::GetAdmin {})
            .unwrap();

        assert_eq!(old_admin, "owner");

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::UpdateAdmin {
                admin: "new_admin".to_string(),
            },
            &[],
        )
        .unwrap();

        let new_admin: String = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::GetAdmin {})
            .unwrap();

        assert_eq!(new_admin, "new_admin");
    }
}
