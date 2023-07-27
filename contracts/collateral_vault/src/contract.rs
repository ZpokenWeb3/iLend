use {
    crate::{
        error::ContractError,
        msg::{InstantiateMsg, ExecuteMsg, QueryMsg},
    },
    cosmwasm_std::{
        coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
        Timestamp, Uint128,
    },
    cw2::set_contract_version,
};
use crate::contract::query::{get_lending_contract, get_margin_contract};
use crate::state::{ADMIN, LENDING, MARGIN_POSITIONS};

const CONTRACT_NAME: &str = "crates.io:collateral_vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    LENDING.save(deps.storage, &msg.lending_contract)?;
    MARGIN_POSITIONS.save(deps.storage, &msg.margin_contract)?;
    ADMIN.save(deps.storage, &msg.admin)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetLendingContract { contract } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );


            LENDING.save(deps.storage, &contract)?;
            Ok(Response::default())
        }
        ExecuteMsg::SetMarginContract { contract } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            MARGIN_POSITIONS.save(deps.storage, &contract)?;

            Ok(Response::default())
        }
    }
}


pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLendingContract => { to_binary(&get_lending_contract(deps, env)?) }
        QueryMsg::GetMarginContract => { to_binary(&get_margin_contract(deps, env)?) }
    }
}

pub mod query {
    use cosmwasm_std::{Deps, Env, StdResult};
    use crate::state::{LENDING, MARGIN_POSITIONS};

    pub fn get_lending_contract(deps: Deps, env: Env) -> StdResult<String> { LENDING.load(deps.storage) }

    pub fn get_margin_contract(deps: Deps, env: Env) -> StdResult<String> { MARGIN_POSITIONS.load(deps.storage) }
}