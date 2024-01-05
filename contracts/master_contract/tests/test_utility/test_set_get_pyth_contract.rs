#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_one_token_setup;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_set_get_pyth_contract() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_of_one_token_setup();

        let initial_pyth_contract: String = "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez".to_string();
        let second_pyth_contract: String = "whatever-address-works".to_string();

        const PRICE_DECIMALS: u32 = 8;
        const PRICE_ETH: u128 = 2000u128 * 10u128.pow(PRICE_DECIMALS);
        const PRICE_ATOM: u128 = 10u128 * 10u128.pow(PRICE_DECIMALS);

        let pyth_contract: String = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPythContract {},
            )
            .unwrap();


        assert_eq!(pyth_contract, initial_pyth_contract);

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetPythContract {
                pyth_contract_addr: second_pyth_contract.clone(),
            },
            &[],
        ).unwrap();

        let pyth_contract_after: String = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPythContract {},
            )
            .unwrap();

        assert_eq!(pyth_contract_after, second_pyth_contract);
    }
}
