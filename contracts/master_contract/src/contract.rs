use crate::contract::query::{get_borrows, get_deposited_amount_in_usd, get_price, get_repay_info, get_supported_tokens, get_tokens_interest_rate_model_params, get_interest_rate};
use crate::msg::{RepayInfo, TokenInfo, TokenInterestRateModelParams};
use crate::state::{PRICES, USER_BORROWED_BALANCE, USER_REPAY_INFO};
use {
    crate::contract::query::get_deposit,
    crate::{
        error::ContractError,
        msg::InstantiateMsg,
        msg::{ExecuteMsg, QueryMsg},
        state::{
            ADMIN,
            SUPPORTED_TOKENS,
            TOKENS_INTEREST_RATE_MODEL_PARAMS,
            USER_DEPOSITED_BALANCE
        },
    },
    cosmwasm_std::{
        coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
        Uint128,
    },
    cw2::set_contract_version,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:master_contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ADMIN.save(deps.storage, &msg.admin)?;

    for tokens in msg.supported_tokens {
        SUPPORTED_TOKENS.save(
            deps.storage,
            tokens.0.clone(),
            &TokenInfo {
                denom: tokens.0,
                name: tokens.1,
                symbol: tokens.2,
                decimals: tokens.3,
            },
        )?;
    }

    for params in msg.tokens_interest_rate_model_params {
        TOKENS_INTEREST_RATE_MODEL_PARAMS.save(
            deps.storage,
            params.0.clone(),
            &TokenInterestRateModelParams {
                denom: params.0,
                min_interest_rate: params.1,
                safe_borrow_max_rate: params.2,
                rate_growth_factor: params.3,
            },
        )?;
    }

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => {
            // underlying balances that are allowing user to send funds along with calling Deposit
            // is operated through cw20 contracts

            if info.funds.is_empty() {
                return Err(ContractError::CustomError {
                    val: "No funds deposited!".to_string(),
                });
            }

            assert_eq!(
                info.funds.len(),
                1,
                "You have to deposit one asset per time"
            );

            let allowed_coin = info.funds.first().unwrap();

            assert!(allowed_coin.amount.u128() > 0);

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, allowed_coin.denom.clone()),
                "There is no such supported token yet"
            );

            let current_balance = get_deposit(
                deps.as_ref(),
                info.sender.to_string(),
                allowed_coin.denom.clone(),
            )
                .unwrap();
            let new_balance = current_balance.balance.u128() + allowed_coin.amount.u128();
            USER_DEPOSITED_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), allowed_coin.denom.clone()),
                &Uint128::from(new_balance),
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::Redeem { denom, amount } => {
            assert!(amount.u128() > 0, "Amount should be a positive number");

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            let current_balance =
                query::get_deposit(deps.as_ref(), info.sender.to_string(), denom.clone())?;

            let amount = amount.u128();

            assert!(
                current_balance.balance.u128() >= amount,
                "The account doesn't have enough digital tokens to do withdraw"
            );

            // let mut deposited_amount_in_usd = get_deposited_amount_in_usd(deps.as_ref(), info.sender.to_string().clone()).unwrap().u128();

            let remaining = current_balance.balance.u128() - amount;

            // let availableToRedeemUSD = sumCollateralBalanceUSD - sumBorrowBalanceUSD / 0.8

            USER_DEPOSITED_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &Uint128::from(remaining),
            )?;

            Ok(Response::new().add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(amount, denom.clone()),
            }))
        }
        ExecuteMsg::Fund {} => {
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
        ExecuteMsg::AddMarkets {
            denom,
            name,
            symbol,
            decimals,
            min_interest_rate,
            safe_borrow_max_rate,
            rate_growth_factor
        } => {
            SUPPORTED_TOKENS.save(
                deps.storage,
                denom.clone(),
                &TokenInfo {
                    denom: denom.clone(),
                    name,
                    symbol,
                    decimals,
                },
            )?;

            TOKENS_INTEREST_RATE_MODEL_PARAMS.save(
                deps.storage,
                denom.clone(),
                &TokenInterestRateModelParams {
                    denom: denom.clone(),
                    min_interest_rate,
                    safe_borrow_max_rate,
                    rate_growth_factor,
                },
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::Borrow { denom, amount } => {
            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            let deposited_amount_in_usd =
                get_deposited_amount_in_usd(deps.as_ref(), info.sender.to_string().clone())
                    .unwrap()
                    .u128();

            // amount available to borrow
            let available_to_borrow_usd = deposited_amount_in_usd * 80u128 / 100u128;

            let available_to_borrow_amount =
                available_to_borrow_usd / get_price(deps.as_ref(), denom.clone()).unwrap().price;

            assert!(
                available_to_borrow_amount >= amount.u128(),
                "User don't have enough deposit to borrow that much"
            );

            let user_borrows = get_borrows(
                deps.as_ref(),
                info.sender.to_string().clone(),
                denom.clone(),
            )
                .unwrap()
                .borrows;

            let new_user_borrows: u128 = user_borrows.u128() + amount.u128();

            USER_BORROWED_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &Uint128::from(new_user_borrows.clone()),
            )?;

            // updating repay info
            let new_user_repay_info = RepayInfo {
                borrowed_amount: Uint128::from(new_user_borrows.clone()),
                accumulated_interest: Uint128::from(new_user_borrows.clone() / 8),
            };

            USER_REPAY_INFO.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &new_user_repay_info,
            )?;

            Ok(Response::new().add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(amount.u128(), denom.clone()),
            }))
        }
        ExecuteMsg::SetPrice { denom, price } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            PRICES.save(deps.storage, denom, &price)?;

            Ok(Response::new())
        }
        ExecuteMsg::Repay {} => {
            if info.funds.is_empty() {
                return Err(ContractError::CustomError {
                    val: "No funds deposited!".to_string(),
                });
            }

            assert_eq!(info.funds.len(), 1, "You have to repay one asset per time");

            let repay_token = info.funds.first().unwrap();

            assert!(repay_token.amount.u128() > 0);

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, repay_token.denom.clone()),
                "There is no such token to repay yet"
            );
            let current_repay_info = get_repay_info(deps.as_ref(), info.sender.to_string().clone(), repay_token.denom.clone()).unwrap_or_default();

            let current_borrows = get_borrows(
                deps.as_ref(),
                info.sender.to_string(),
                repay_token.denom.clone(),
            )
                .unwrap();


            if repay_token.amount.u128() < current_repay_info.accumulated_interest.u128() {
                let new_user_repay_info = RepayInfo {
                    borrowed_amount: current_borrows.borrows,
                    accumulated_interest: Uint128::from(current_repay_info.accumulated_interest.u128() - repay_token.amount.u128()),
                };

                USER_REPAY_INFO.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &new_user_repay_info,
                )?;
                Ok(Response::default())
            } else if repay_token.amount.u128() <= current_repay_info.accumulated_interest.u128() + current_borrows.borrows.u128() {
                let new_borrow_amount = current_borrows.borrows.u128() - (repay_token.amount.u128() - current_repay_info.accumulated_interest.u128());
                let new_user_repay_info = RepayInfo {
                    borrowed_amount: Uint128::from(new_borrow_amount),
                    accumulated_interest: Default::default(),
                };

                USER_REPAY_INFO.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &new_user_repay_info,
                )?;

                USER_BORROWED_BALANCE.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &Uint128::from(new_borrow_amount),
                )?;
                Ok(Response::default())
            } else {
                USER_REPAY_INFO.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &RepayInfo::default(),
                )?;

                USER_BORROWED_BALANCE.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &Uint128::zero(),
                )?;
                // send back if all borrowed was repaid
                let remaining = repay_token.amount.u128() - current_repay_info.accumulated_interest.u128() - current_borrows.borrows.u128();

                Ok(Response::new().add_message(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: coins(remaining, repay_token.denom.clone()),
                }))
            }
        }
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposit { address, denom } => to_binary(&get_deposit(deps, address, denom)?),
        QueryMsg::GetPrice { denom } => to_binary(&get_price(deps, denom)?),
        QueryMsg::GetBorrows { address, denom } => {
            to_binary(&query::get_borrows(deps, address, denom)?)
        }
        QueryMsg::GetRepayInfo { address, denom } => {
            to_binary(&query::get_repay_info(deps, address, denom)?)
        }
        QueryMsg::GetSupportedTokens {} => to_binary(&get_supported_tokens(deps)?),
        QueryMsg::GetTokensInterestRateModelParams {} => to_binary(&get_tokens_interest_rate_model_params(deps)?),
        QueryMsg::GetInterestRate { denom } => to_binary(&get_interest_rate(deps, denom)?),
    }
}

pub mod query {
    use super::*;
    use crate::msg::{
        GetBalanceResponse,
        GetBorrowsResponse,
        GetPriceResponse,
        GetSupportedTokensResponse,
        GetTokensInterestRateModelParamsResponse,
        RepayInfo,
        GetInterestRateResponse
    };
    use cosmwasm_std::Order;

    pub fn get_deposit(deps: Deps, user: String, denom: String) -> StdResult<GetBalanceResponse> {
        let balance = USER_DEPOSITED_BALANCE
            .load(deps.storage, (user, denom))
            .unwrap_or_else(|_| Uint128::zero());

        Ok(GetBalanceResponse { balance })
    }

    pub fn get_borrows(deps: Deps, user: String, denom: String) -> StdResult<GetBorrowsResponse> {
        let borrows = USER_BORROWED_BALANCE
            .load(deps.storage, (user, denom))
            .unwrap_or_else(|_| Uint128::zero());

        Ok(GetBorrowsResponse { borrows })
    }

    pub fn get_price(deps: Deps, denom: String) -> StdResult<GetPriceResponse> {
        let price = PRICES.load(deps.storage, denom).unwrap_or_else(|_| 0u128);

        Ok(GetPriceResponse { price })
    }

    pub fn get_supported_tokens(deps: Deps) -> StdResult<GetSupportedTokensResponse> {
        let mut result: Vec<TokenInfo> = vec![];

        let all: StdResult<Vec<_>> = SUPPORTED_TOKENS
            .range(deps.storage, None, None, Order::Ascending)
            .collect();
        for el in all.unwrap() {
            result.push(el.1)
        }

        Ok(GetSupportedTokensResponse {
            supported_tokens: result,
        })
    }

    pub fn get_tokens_interest_rate_model_params(deps: Deps) -> StdResult<GetTokensInterestRateModelParamsResponse> {
        let mut result: Vec<TokenInterestRateModelParams> = vec![];

        let all: StdResult<Vec<_>> = TOKENS_INTEREST_RATE_MODEL_PARAMS
            .range(deps.storage, None, None, Order::Ascending)
            .collect();
        for el in all.unwrap() {
            result.push(el.1)
        }

        Ok(GetTokensInterestRateModelParamsResponse {
            tokens_interest_rate_model_params: result,
        })
    }

    pub fn get_interest_rate(deps: Deps, denom: String) -> StdResult<GetInterestRateResponse> {
        let utilization_rate = 40 * 10u128.pow(15); // mock utilization_rate == 40%
        const UTILIZATION_LIMIT: u128 = 80 * 10u128.pow(15); // 80%
        const HUNDRED: u128 = 100 * 10u128.pow(15);

        let min_interest_rate = TOKENS_INTEREST_RATE_MODEL_PARAMS.load(deps.storage, denom.clone()).unwrap().min_interest_rate;
        let safe_borrow_max_rate = TOKENS_INTEREST_RATE_MODEL_PARAMS.load(deps.storage, denom.clone()).unwrap().safe_borrow_max_rate;
        let rate_growth_factor = TOKENS_INTEREST_RATE_MODEL_PARAMS.load(deps.storage, denom.clone()).unwrap().rate_growth_factor;

        if utilization_rate <= UTILIZATION_LIMIT {
            Ok(GetInterestRateResponse {
                interest_rate: min_interest_rate + utilization_rate * (safe_borrow_max_rate - min_interest_rate) / UTILIZATION_LIMIT
            })
        } else {
            Ok(GetInterestRateResponse {
                interest_rate: safe_borrow_max_rate + rate_growth_factor * (utilization_rate - UTILIZATION_LIMIT)  / (HUNDRED - UTILIZATION_LIMIT)
            })
        }
    }

    pub fn get_token_decimal(deps: Deps, denom: String) -> StdResult<Uint128> {
        // contract only inner call, so there is no need to parse non-existent token denom
        Ok(Uint128::from(
            SUPPORTED_TOKENS.load(deps.storage, denom).unwrap().decimals,
        ))
    }

    pub fn get_deposited_amount_in_usd(deps: Deps, user: String) -> StdResult<Uint128> {
        let mut deposited_amount_in_usd = 0u128;
        // users deposited amount in USD
        for token in get_supported_tokens(deps).unwrap().supported_tokens {
            let users_deposit = get_deposit(deps, user.clone(), token.denom.clone())
                .unwrap()
                .balance
                .u128();

            if users_deposit != 0 {
                deposited_amount_in_usd +=
                    users_deposit * get_price(deps, token.denom).unwrap().price;
            }
        }

        Ok(Uint128::from(deposited_amount_in_usd))
    }

    pub fn get_repay_info(deps: Deps, user: String, denom: String) -> StdResult<RepayInfo> {
        Ok(
            USER_REPAY_INFO.load(deps.storage, (user, denom)).unwrap_or_default()
        )
    }
}
