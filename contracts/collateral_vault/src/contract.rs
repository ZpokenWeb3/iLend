use crate::contract::query::{get_lending_contract, get_margin_contract};
use crate::state::{ADMIN, LENDING_CONTRACT, MARGIN_POSITIONS_CONTRACT};
use {
    crate::{
        error::ContractError,
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    },
    cosmwasm_std::{
        coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    },
    cw2::set_contract_version,
};

const CONTRACT_NAME: &str = "crates.io:collateral_vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    LENDING_CONTRACT.save(deps.storage, &msg.lending_contract)?;
    MARGIN_POSITIONS_CONTRACT.save(deps.storage, &msg.margin_contract)?;
    ADMIN.save(deps.storage, &msg.admin)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetLendingContract { contract } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            LENDING_CONTRACT.save(deps.storage, &contract)?;
            Ok(Response::default())
        }
        ExecuteMsg::SetMarginPositionsContract { contract } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            MARGIN_POSITIONS_CONTRACT.save(deps.storage, &contract)?;

            Ok(Response::default())
        }
        ExecuteMsg::RedeemFromVaultContract {
            denom,
            amount,
            user,
        } => {
            assert_eq!(
                info.sender.to_string(),
                LENDING_CONTRACT.load(deps.storage).unwrap(),
                "This functionality is allowed for lending contract only"
            );

            Ok(Response::new().add_message(BankMsg::Send {
                to_address: user,
                amount: coins(amount.u128(), denom),
            }))
        }
        ExecuteMsg::RedeemFromVaultContractMargin {
            denom,
            amount,
            user,
        } => {
            assert_eq!(
                info.sender.to_string(),
                MARGIN_POSITIONS_CONTRACT.load(deps.storage).unwrap(),
                "This functionality is allowed for lending contract only"
            );

            Ok(Response::new().add_message(BankMsg::Send {
                to_address: user,
                amount: coins(amount.u128(), denom),
            }))
        }
        ExecuteMsg::BorrowFromVaultContract {
            denom,
            amount,
            user,
        } => {
            assert_eq!(
                info.sender.to_string(),
                LENDING_CONTRACT.load(deps.storage).unwrap(),
                "This functionality is allowed for lending contract only"
            );

            Ok(Response::new().add_message(BankMsg::Send {
                to_address: user,
                amount: coins(amount.u128(), denom),
            }))
        }
        ExecuteMsg::Fund {} => {
            // Admin-only functionality for funding contract with reserves
            // to be able to operate borrows and repayments
            if info.funds.is_empty() {
                return Err(ContractError::CustomError {
                    val: "No funds deposited!".to_string(),
                });
            }

            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            Ok(Response::default())
        }
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLendingContract => to_binary(&get_lending_contract(deps)?),
        QueryMsg::GetMarginContract => to_binary(&get_margin_contract(deps)?),
    }
}

pub mod query {
    use crate::state::{LENDING_CONTRACT, MARGIN_POSITIONS_CONTRACT};
    use cosmwasm_std::{Deps, StdResult};

    pub fn get_lending_contract(deps: Deps) -> StdResult<String> {
        LENDING_CONTRACT.load(deps.storage)
    }

    pub fn get_margin_contract(deps: Deps) -> StdResult<String> {
        MARGIN_POSITIONS_CONTRACT.load(deps.storage)
    }
}
