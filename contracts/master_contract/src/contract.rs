use crate::contract::query::{
    get_available_liquidity_by_token, get_available_to_borrow, get_available_to_redeem,
    get_borrow_amount_with_interest, get_contract_balance_by_token, get_current_liquidity_index_ln,
    get_interest_rate, get_liquidity_index_last_update, get_liquidity_rate, get_mm_token_price,
    get_price, get_supported_tokens, get_token_decimal, get_tokens_interest_rate_model_params,
    get_total_borrow_data, get_total_borrowed_by_token_usd, get_total_deposited_by_token_usd,
    get_total_reserves_by_token, get_user_borrowed_usd, get_user_borrowing_info,
    get_user_deposited_usd, get_utilization_rate_by_token, user_deposit_as_collateral,
};
use crate::msg::{
    LiquidityIndexData, TokenInfo, TokenInterestRateModelParams, TotalBorrowData, UserBorrowingInfo,
};
use crate::state::{
    LIQUIDITY_INDEX_DATA,
    PRICES,
    TOTAL_BORROW_DATA,
    USER_BORROWED_BALANCE,
    USER_BORROWING_INFO,
    USER_DEPOSIT_AS_COLLATERAL
};
use rust_decimal::prelude::{Decimal, MathematicalOps};
// use rust_decimal::Decimal;

use std::ops::{Add, Div, Mul};
use {
    crate::contract::query::get_deposit,
    crate::{
        error::ContractError,
        msg::InstantiateMsg,
        msg::{ExecuteMsg, QueryMsg},
        state::{
            ADMIN, SUPPORTED_TOKENS, TOKENS_INTEREST_RATE_MODEL_PARAMS, USER_MM_TOKEN_BALANCE,
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

const PERCENT_DECIMALS: u32 = 5;
const HUNDRED_PERCENT: u128 = 100 * 10u128.pow(PERCENT_DECIMALS);
const UTILIZATION_LIMIT: u128 = 80 * 10u128.pow(PERCENT_DECIMALS); // 80%

const INTEREST_RATE_DECIMALS: u32 = 18;
const INTEREST_RATE_MULTIPLIER: u128 = 10u128.pow(INTEREST_RATE_DECIMALS);
const HUNDRED: u128 = 100;
const YEAR_IN_SECONDS: u128 = 31536000; // 365 days

const USD_DECIMALS: u32 = 8;
// const USD_MULTIPLIER: u128 = 10u128.pow(USD_DECIMALS);

pub trait DecimalExt {
    fn to_u128_with_decimals(&self, decimals: u32) -> StdResult<u128>;
}

impl DecimalExt for Decimal {
    fn to_u128_with_decimals(&self, decimals: u32) -> StdResult<u128> {
        let s = self.to_string();
        let (left, right) = s.split_once(".").unwrap_or((&s, ""));
        let mut right = right.to_string();
        let right_len = right.len() as u32;
        if right_len > decimals {
            right.truncate(decimals.try_into().unwrap());
        } else if right_len < decimals {
            let zeroes = decimals - right_len;
            right.push_str(&"0".repeat(zeroes.try_into().unwrap()));
        }
        let s = format!("{}{}", left, right);
        Ok(s.parse::<u128>().unwrap_or(0))
    }
}

pub fn instantiate(
    deps: DepsMut,
    env: Env,
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
                denom: tokens.0.clone(),
                name: tokens.1,
                symbol: tokens.2,
                decimals: tokens.3,
            },
        )?;

        TOTAL_BORROW_DATA.save(
            deps.storage,
            tokens.0.clone(),
            &TotalBorrowData {
                denom: tokens.0.clone(),
                total_borrowed_amount: 0u128,
                expected_annual_interest_income: 0u128,
            },
        )?;

        LIQUIDITY_INDEX_DATA.save(
            deps.storage,
            tokens.0.clone(),
            &LiquidityIndexData {
                denom: tokens.0.clone(),
                liquidity_index_ln: 0u128,
                timestamp: env.block.time,
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
    mut deps: DepsMut,
    env: Env,
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

            let deposited_token = info.funds.first().unwrap();
            let deposited_token_amount = deposited_token.amount.u128();

            assert!(deposited_token_amount > 0);

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, deposited_token.denom.clone()),
                "There is no such supported token yet"
            );

            execute_update_liquidity_index_data(
                &mut deps,
                env.clone(),
                deposited_token.denom.clone(),
            )?;

            let token_decimals = get_token_decimal(deps.as_ref(), deposited_token.denom.clone())
                .unwrap()
                .u128() as u32;

            let mm_token_price =
                get_mm_token_price(deps.as_ref(), env.clone(), deposited_token.denom.clone())
                    .unwrap()
                    .u128();

            let deposited_mm_token_amount =
                Decimal::from_i128_with_scale(deposited_token_amount as i128, token_decimals)
                    .div(Decimal::from_i128_with_scale(
                        mm_token_price as i128,
                        token_decimals,
                    ))
                    .to_u128_with_decimals(token_decimals)
                    .unwrap();

            let user_current_mm_token_balance = USER_MM_TOKEN_BALANCE
                .load(
                    deps.storage,
                    (info.sender.to_string(), deposited_token.denom.clone()),
                )
                .unwrap_or_else(|_| Uint128::zero());

            let new_user_mm_token_balance =
                user_current_mm_token_balance.u128() + deposited_mm_token_amount;

            USER_MM_TOKEN_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), deposited_token.denom.clone()),
                &Uint128::from(new_user_mm_token_balance),
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::Redeem { denom, amount } => {
            assert!(amount.u128() > 0, "Amount should be a positive number");

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            execute_update_liquidity_index_data(&mut deps, env.clone(), denom.clone())?;

            let current_balance = query::get_deposit(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                denom.clone(),
            )?;

            let amount = amount.u128();

            assert!(
                current_balance.balance.u128() >= amount,
                "The account doesn't have enough digital tokens to do withdraw"
            );

            let remaining = current_balance.balance.u128() - amount;

            let token_decimals = get_token_decimal(deps.as_ref(), denom.clone())
                .unwrap()
                .u128() as u32;

            let mm_token_price = get_mm_token_price(deps.as_ref(), env.clone(), denom.clone())
                .unwrap()
                .u128();

            let new_user_mm_token_balance =
                Decimal::from_i128_with_scale(remaining as i128, token_decimals)
                    .div(Decimal::from_i128_with_scale(
                        mm_token_price as i128,
                        token_decimals,
                    ))
                    .to_u128_with_decimals(token_decimals)
                    .unwrap();

            USER_MM_TOKEN_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &Uint128::from(new_user_mm_token_balance),
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
            rate_growth_factor,
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

            TOTAL_BORROW_DATA.save(
                deps.storage,
                denom.clone(),
                &TotalBorrowData {
                    denom: denom.clone(),
                    total_borrowed_amount: 0u128,
                    expected_annual_interest_income: 0u128,
                },
            )?;

            LIQUIDITY_INDEX_DATA.save(
                deps.storage,
                denom.clone(),
                &LiquidityIndexData {
                    denom: denom.clone(),
                    liquidity_index_ln: 0u128,
                    timestamp: env.block.time,
                },
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::Borrow { denom, amount } => {
            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            let available_to_borrow_amount = get_available_to_borrow(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                denom.clone(),
            )
            .unwrap()
            .u128();

            assert!(
                available_to_borrow_amount >= amount.u128(),
                "User don't have enough deposit to borrow that much"
            );

            assert!(
                get_contract_balance_by_token(deps.as_ref(), env.clone(), denom.clone())
                    .unwrap()
                    .u128()
                    >= amount.u128()
            );

            execute_update_liquidity_index_data(&mut deps, env.clone(), denom.clone())?;

            let borrow_amount_with_interest = get_borrow_amount_with_interest(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                denom.clone(),
            )
            .unwrap()
            .amount
            .u128();

            let user_borrowing_info = get_user_borrowing_info(
                deps.as_ref(),
                info.sender.to_string().clone(),
                denom.clone(),
            )
            .unwrap_or_default();

            let new_user_borrow_amount: u128 = borrow_amount_with_interest + amount.u128();

            let current_interest_rate =
                get_interest_rate(deps.as_ref(), env.clone(), denom.clone())
                    .unwrap()
                    .u128();

            let borrowed_token_decimals = get_token_decimal(deps.as_ref(), denom.clone())
                .unwrap()
                .u128() as u32;

            let average_interest_rate = (Decimal::from_i128_with_scale(
                borrow_amount_with_interest as i128,
                borrowed_token_decimals,
            )
            .mul(Decimal::from_i128_with_scale(
                user_borrowing_info.average_interest_rate.u128() as i128,
                INTEREST_RATE_DECIMALS,
            ))
            .add(
                Decimal::from_i128_with_scale(amount.u128() as i128, borrowed_token_decimals).mul(
                    Decimal::from_i128_with_scale(
                        current_interest_rate as i128,
                        INTEREST_RATE_DECIMALS,
                    ),
                ),
            ))
            .div(Decimal::from_i128_with_scale(
                new_user_borrow_amount as i128,
                borrowed_token_decimals,
            ))
            .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
            .unwrap();

            // updating user borrowing info
            let new_user_borrowing_info = UserBorrowingInfo {
                borrowed_amount: Uint128::from(new_user_borrow_amount.clone()),
                average_interest_rate: Uint128::from(average_interest_rate),
                timestamp: env.block.time,
            };

            let total_borrow_data =
                get_total_borrow_data(deps.as_ref(), denom.clone()).unwrap_or_default();

            let expected_annual_interest_income = total_borrow_data.expected_annual_interest_income
                - Decimal::from_i128_with_scale(
                    user_borrowing_info.borrowed_amount.u128() as i128,
                    borrowed_token_decimals,
                )
                .mul(Decimal::from_i128_with_scale(
                    (user_borrowing_info.average_interest_rate.u128() / HUNDRED) as i128,
                    INTEREST_RATE_DECIMALS,
                ))
                .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                .unwrap()
                + Decimal::from_i128_with_scale(
                    new_user_borrow_amount as i128,
                    borrowed_token_decimals,
                )
                .mul(Decimal::from_i128_with_scale(
                    (average_interest_rate / HUNDRED) as i128,
                    INTEREST_RATE_DECIMALS,
                ))
                .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                .unwrap();

            // updating total borrow data
            let new_total_borrow_data = TotalBorrowData {
                denom: denom.clone(),
                total_borrowed_amount: total_borrow_data.total_borrowed_amount
                    - user_borrowing_info.borrowed_amount.u128()
                    + new_user_borrow_amount.clone(),
                expected_annual_interest_income: expected_annual_interest_income,
            };

            USER_BORROWING_INFO.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &new_user_borrowing_info,
            )?;

            TOTAL_BORROW_DATA.save(deps.storage, denom.clone(), &new_total_borrow_data)?;

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
        ExecuteMsg::ToggleCollateralSetting { denom } => {
            let use_user_deposit_as_collateral = user_deposit_as_collateral(
                deps.as_ref(),
                info.sender.to_string(),
                denom.clone(),
            )
                .unwrap();

            if use_user_deposit_as_collateral == true {
                let available_to_redeem = get_available_to_redeem(
                    deps.as_ref(),
                    env.clone(),
                    info.sender.to_string(),
                    denom.clone(),
                )
                    .unwrap()
                    .u128();

                let current_user_deposit = get_deposit(
                    deps.as_ref(),
                    env.clone(),
                    info.sender.to_string(),
                    denom.clone(),
                )
                    .unwrap()
                    .balance
                    .u128();

                assert!(
                    available_to_redeem >= current_user_deposit,
                    "The collateral has already using to collateralise the borrowing. Not enough available balance"
                );
            }

            USER_DEPOSIT_AS_COLLATERAL.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &!use_user_deposit_as_collateral,
            )?;

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
            let current_borrowing_info = get_user_borrowing_info(
                deps.as_ref(),
                info.sender.to_string().clone(),
                repay_token.denom.clone(),
            )
            .unwrap_or_default();

            execute_update_liquidity_index_data(&mut deps, env.clone(), repay_token.denom.clone())?;

            let borrow_amount_with_interest = get_borrow_amount_with_interest(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                repay_token.denom.clone(),
            )
            .unwrap()
            .amount;

            if repay_token.amount.u128() < borrow_amount_with_interest.u128() {
                let new_user_borrowing_info = UserBorrowingInfo {
                    borrowed_amount: Uint128::from(
                        borrow_amount_with_interest.u128() - repay_token.amount.u128(),
                    ),
                    average_interest_rate: current_borrowing_info.average_interest_rate,
                    timestamp: env.block.time,
                };

                USER_BORROWING_INFO.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &new_user_borrowing_info,
                )?;

                USER_BORROWED_BALANCE.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &Uint128::from(borrow_amount_with_interest.u128() - repay_token.amount.u128()),
                )?;

                Ok(Response::default())
            } else if repay_token.amount.u128() > borrow_amount_with_interest.u128() {
                let remaining = repay_token.amount.u128() - borrow_amount_with_interest.u128();

                USER_BORROWING_INFO.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &UserBorrowingInfo::default(),
                )?;

                USER_BORROWED_BALANCE.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &Uint128::zero(),
                )?;

                Ok(Response::new().add_message(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: coins(remaining, repay_token.denom.clone()),
                }))
            } else {
                USER_BORROWING_INFO.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &UserBorrowingInfo::default(),
                )?;

                USER_BORROWED_BALANCE.save(
                    deps.storage,
                    (info.sender.to_string(), repay_token.denom.clone()),
                    &Uint128::zero(),
                )?;
                Ok(Response::default())
            }
        }
    }
}

pub fn execute_update_liquidity_index_data(
    deps: &mut DepsMut,
    env: Env,
    denom: String,
) -> StdResult<Response> {
    let current_liquidity_index_ln =
        get_current_liquidity_index_ln(deps.as_ref(), env.clone(), denom.clone())
            .unwrap()
            .u128();

    let new_liquidity_index_data = LiquidityIndexData {
        denom: denom.clone(),
        liquidity_index_ln: current_liquidity_index_ln,
        timestamp: env.block.time,
    };

    LIQUIDITY_INDEX_DATA.save(deps.storage, denom.clone(), &new_liquidity_index_data)?;

    Ok(Response::new().add_attribute("liquidity_index", format!("{}", env.block.time)))
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposit { address, denom } => {
            to_binary(&get_deposit(deps, env, address, denom)?)
        }
        QueryMsg::UserDepositAsCollateral { address, denom } => {
            to_binary(&user_deposit_as_collateral(deps, address, denom)?)
        }
        QueryMsg::GetPrice { denom } => to_binary(&get_price(deps, denom)?),
        QueryMsg::GetBorrowAmountWithInterest { address, denom } => to_binary(
            &query::get_borrow_amount_with_interest(deps, env, address, denom)?,
        ),
        QueryMsg::GetUserBorrowingInfo { address, denom } => {
            to_binary(&query::get_user_borrowing_info(deps, address, denom)?)
        }
        QueryMsg::GetTotalBorrowData { denom } => {
            to_binary(&query::get_total_borrow_data(deps, denom)?)
        }
        QueryMsg::GetSupportedTokens {} => to_binary(&get_supported_tokens(deps)?),
        QueryMsg::GetTokensInterestRateModelParams {} => {
            to_binary(&get_tokens_interest_rate_model_params(deps)?)
        }
        QueryMsg::GetInterestRate { denom } => to_binary(&get_interest_rate(deps, env, denom)?),
        QueryMsg::GetLiquidityRate { denom } => to_binary(&get_liquidity_rate(deps, env, denom)?),
        QueryMsg::GetCurrentLiquidityIndexLn { denom } => {
            to_binary(&get_current_liquidity_index_ln(deps, env, denom)?)
        }
        QueryMsg::GetMmTokenPrice { denom } => to_binary(&get_mm_token_price(deps, env, denom)?),
        QueryMsg::GetUserDepositedUsd { address } => {
            to_binary(&get_user_deposited_usd(deps, env, address)?)
        }
        QueryMsg::GetUserBorrowedUsd { address } => {
            to_binary(&get_user_borrowed_usd(deps, env, address)?)
        }
        QueryMsg::GetContractBalance { denom } => {
            to_binary(&get_contract_balance_by_token(deps, env, denom)?)
        }
        QueryMsg::GetAvailableToBorrow { address, denom } => {
            to_binary(&get_available_to_borrow(deps, env, address, denom)?)
        }
        QueryMsg::GetAvailableToRedeem { address, denom } => {
            to_binary(&get_available_to_redeem(deps, env, address, denom)?)
        }
        QueryMsg::GetTotalReservesByToken { denom } => {
            to_binary(&get_total_reserves_by_token(deps, env, denom)?)
        }
        QueryMsg::GetTotalDepositedByToken { denom } => {
            to_binary(&get_total_deposited_by_token_usd(deps, env, denom)?)
        }
        QueryMsg::GetTotalBorrowedByToken { denom } => {
            to_binary(&get_total_borrowed_by_token_usd(deps, denom)?)
        }
        QueryMsg::GetAvailableLiquidityByToken { denom } => {
            to_binary(&get_available_liquidity_by_token(deps, env, denom)?)
        }
        QueryMsg::GetUtilizationRateByToken { denom } => {
            to_binary(&get_utilization_rate_by_token(deps, env, denom)?)
        }
        QueryMsg::GetLiquidityIndexLastUpdate { denom } => {
            to_binary(&get_liquidity_index_last_update(deps, denom)?)
        }
    }
}

pub mod query {
    use super::*;
    //     use std::fmt::Debug;
    use std::ops::Mul;

    //     use std::str::FromStr;

    use crate::msg::{
        GetBalanceResponse, GetBorrowAmountWithInterestResponse,
        GetSupportedTokensResponse, GetTokensInterestRateModelParamsResponse,
        GetUserBorrowedUsdResponse, GetUserDepositedUsdResponse, TotalBorrowData,
        UserBorrowingInfo,
    };
    use cosmwasm_std::{Coin, Order};
    //     use near_sdk::json_types::U128;
    //     use rust_decimal::prelude::ToPrimitive;

    pub fn get_deposit(
        deps: Deps,
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<GetBalanceResponse> {
        let token_decimals = get_token_decimal(deps, denom.clone())
            .unwrap()
            .u128() as u32;

        let user_mm_token_balance = USER_MM_TOKEN_BALANCE
            .load(deps.storage, (user, denom.clone()))
            .unwrap_or_else(|_| Uint128::zero());

        let mm_token_price = get_mm_token_price(deps.clone(), env.clone(), denom.clone())
            .unwrap()
            .u128();

        let user_token_balance =
            Decimal::from_i128_with_scale(user_mm_token_balance.u128() as i128, token_decimals)
                .mul(Decimal::from_i128_with_scale(
                    mm_token_price as i128,
                    token_decimals,
                ))
                .to_u128_with_decimals(token_decimals)
                .unwrap();

        Ok(GetBalanceResponse {
            balance: Uint128::from(user_token_balance),
        })
    }

    pub fn user_deposit_as_collateral(
        deps: Deps,
        user: String,
        denom: String,
    ) -> StdResult<bool> {
        let use_user_deposit_as_collateral = USER_DEPOSIT_AS_COLLATERAL
            .load(
                deps.storage,
                (user, denom.clone()),
            )
            .unwrap_or_default();

        Ok(use_user_deposit_as_collateral)
    }

    pub fn get_borrow_amount_with_interest(
        deps: Deps,
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<GetBorrowAmountWithInterestResponse> {
        let current_borrowing_info =
            get_user_borrowing_info(deps.clone(), user.clone(), denom.clone()).unwrap_or_default();

        let token_decimals = get_token_decimal(deps, denom.clone())
            .unwrap()
            .u128() as u32;

        let current_borrowed_amount = Decimal::from_i128_with_scale(
            current_borrowing_info.borrowed_amount.u128() as i128,
            token_decimals,
        );

        let base = Decimal::from_i128_with_scale(
            (current_borrowing_info.average_interest_rate.u128() / HUNDRED
                + INTEREST_RATE_MULTIPLIER) as i128,
            INTEREST_RATE_DECIMALS,
        );

        let exponent = Decimal::from_i128_with_scale(
            ((env.block.time.seconds() - current_borrowing_info.timestamp.seconds()) as u128
                * INTEREST_RATE_MULTIPLIER
                / YEAR_IN_SECONDS) as i128,
            INTEREST_RATE_DECIMALS,
        );

        let borrow_amount_with_interest = current_borrowed_amount
            .mul(base.powd(exponent))
            .to_u128_with_decimals(token_decimals)
            .unwrap();

        Ok(GetBorrowAmountWithInterestResponse {
            amount: Uint128::from(borrow_amount_with_interest),
            base: Uint128::from(base.to_u128_with_decimals(INTEREST_RATE_DECIMALS).unwrap()),
            exponent: Uint128::from(
                exponent
                    .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                    .unwrap(),
            ),
            average_interest_rate: Uint128::from(current_borrowing_info.average_interest_rate),
        })
    }

    pub fn get_liquidity_rate(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let expected_annual_interest_income = TOTAL_BORROW_DATA
            .load(deps.storage, denom.clone())
            .unwrap()
            .expected_annual_interest_income;

        let reserves_by_token = get_total_reserves_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        let token_decimals = get_token_decimal(deps, denom.clone())
            .unwrap()
            .u128() as u32;

        if reserves_by_token == 0 {
            Ok(Uint128::from(0u128))
        } else {
            let liquidity_rate = Decimal::from_i128_with_scale(
                expected_annual_interest_income as i128,
                INTEREST_RATE_DECIMALS,
            )
            .mul(Decimal::from_i128_with_scale(HUNDRED as i128, 0u32))
            .div(Decimal::from_i128_with_scale(
                reserves_by_token as i128,
                token_decimals,
            ))
            .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
            .unwrap();

            Ok(Uint128::from(liquidity_rate))
        }
    }

    pub fn get_current_liquidity_index_ln(
        deps: Deps,
        env: Env,
        denom: String,
    ) -> StdResult<Uint128> {
        let liquidity_rate = get_liquidity_rate(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        let liquidity_index_last_update = LIQUIDITY_INDEX_DATA
            .load(deps.storage, denom.clone())
            .unwrap()
            .timestamp;

        let liquidity_index_ln = LIQUIDITY_INDEX_DATA
            .load(deps.storage, denom.clone())
            .unwrap()
            .liquidity_index_ln;

        let new_liquidity_index_ln = (env
            .block
            .time
            .seconds()
            .checked_sub(liquidity_index_last_update.seconds())
            .unwrap_or_default()) as u128
            * Decimal::from_i128_with_scale(
                (liquidity_rate / HUNDRED + INTEREST_RATE_MULTIPLIER) as i128,
                INTEREST_RATE_DECIMALS,
            )
            .ln()
            .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
            .unwrap()
            / YEAR_IN_SECONDS
            + liquidity_index_ln;

        Ok(Uint128::from(new_liquidity_index_ln))
    }

    pub fn get_liquidity_index_last_update(deps: Deps, denom: String) -> StdResult<Uint128> {
        Ok(Uint128::from(
            LIQUIDITY_INDEX_DATA
                .load(deps.storage, denom.clone())
                .unwrap()
                .liquidity_index_ln,
        ))
    }

    // number of tokens that correspond to one mmToken
    pub fn get_mm_token_price(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let token_decimals = get_token_decimal(deps, denom.clone())
            .unwrap()
            .u128() as u32;

        let current_liquidity_index_ln =
            get_current_liquidity_index_ln(deps, env.clone(), denom.clone())
                .unwrap()
                .u128();

        let mm_token_price = Decimal::from_i128_with_scale(
            current_liquidity_index_ln as i128,
            INTEREST_RATE_DECIMALS,
        )
        .exp()
        .to_u128_with_decimals(token_decimals)
        .unwrap_or_default();

        Ok(Uint128::from(mm_token_price))
    }

    pub fn get_price(deps: Deps, denom: String) -> StdResult<Uint128> {
        let price = PRICES.load(deps.storage, denom).unwrap_or(0u128);

        Ok(Uint128::from(price))
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

    pub fn get_tokens_interest_rate_model_params(
        deps: Deps,
    ) -> StdResult<GetTokensInterestRateModelParamsResponse> {
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

    pub fn get_interest_rate(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let utilization_rate = get_utilization_rate_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        let min_interest_rate = TOKENS_INTEREST_RATE_MODEL_PARAMS
            .load(deps.storage, denom.clone())
            .unwrap()
            .min_interest_rate;
        let safe_borrow_max_rate = TOKENS_INTEREST_RATE_MODEL_PARAMS
            .load(deps.storage, denom.clone())
            .unwrap()
            .safe_borrow_max_rate;
        let rate_growth_factor = TOKENS_INTEREST_RATE_MODEL_PARAMS
            .load(deps.storage, denom.clone())
            .unwrap()
            .rate_growth_factor;

        if utilization_rate <= UTILIZATION_LIMIT {
            Ok(Uint128::from(
                min_interest_rate
                    + utilization_rate * (safe_borrow_max_rate - min_interest_rate)
                        / UTILIZATION_LIMIT,
            ))
        } else {
            Ok(Uint128::from(
                safe_borrow_max_rate
                    + rate_growth_factor * (utilization_rate - UTILIZATION_LIMIT)
                        / (HUNDRED_PERCENT - UTILIZATION_LIMIT),
            ))
        }
    }

    pub fn get_token_decimal(deps: Deps, denom: String) -> StdResult<Uint128> {
        // contract only inner call, so there is no need to parse non-existent token denom
        Ok(Uint128::from(
            SUPPORTED_TOKENS.load(deps.storage, denom).unwrap().decimals,
        ))
    }

    pub fn get_user_borrowing_info(
        deps: Deps,
        user: String,
        denom: String,
    ) -> StdResult<UserBorrowingInfo> {
        Ok(USER_BORROWING_INFO
            .load(deps.storage, (user, denom))
            .unwrap_or_default())
    }

    pub fn get_total_borrow_data(deps: Deps, denom: String) -> StdResult<TotalBorrowData> {
        Ok(TOTAL_BORROW_DATA
            .load(deps.storage, denom)
            .unwrap_or_default())
    }

    pub fn get_user_deposited_usd(
        deps: Deps,
        env: Env,
        user: String,
    ) -> StdResult<GetUserDepositedUsdResponse> {
        let mut user_deposited_usd = 0u128;

        for tokens in get_supported_tokens(deps).unwrap().supported_tokens {
            let user_deposit = get_deposit(deps, env.clone(), user.clone(), tokens.denom.clone())
                .unwrap()
                .balance
                .u128();

            let token_decimals = get_token_decimal(deps, tokens.denom.clone())
                .unwrap()
                .u128() as u32;

            let price = get_price(deps, tokens.denom.clone())
                .unwrap()
                .u128();

            user_deposited_usd += 
                Decimal::from_i128_with_scale(
                    user_deposit as i128,
                    token_decimals,
                )
                    .mul(Decimal::from_i128_with_scale(
                        price as i128,
                        USD_DECIMALS,
                    ))
                    .to_u128_with_decimals(USD_DECIMALS)
                    .unwrap()
        }

        Ok(GetUserDepositedUsdResponse {
            user_deposited_usd: Uint128::from(user_deposited_usd),
        })
    }

    pub fn get_user_borrowed_usd(
        deps: Deps,
        env: Env,
        user: String,
    ) -> StdResult<GetUserBorrowedUsdResponse> {
        let mut user_borrowed_usd = 0u128;
        for tokens in get_supported_tokens(deps).unwrap().supported_tokens {
            let user_borrow = get_borrow_amount_with_interest(
                deps,
                env.clone(),
                user.clone(),
                tokens.denom.clone(),
            )
                .unwrap()
                .amount
                .u128();

            let token_decimals = get_token_decimal(deps, tokens.denom.clone())
                .unwrap()
                .u128() as u32;

            let price = get_price(deps, tokens.denom.clone())
                .unwrap()
                .u128();

            user_borrowed_usd += 
                Decimal::from_i128_with_scale(
                    user_borrow as i128,
                    token_decimals,
                )
                    .mul(Decimal::from_i128_with_scale(
                        price as i128,
                        USD_DECIMALS,
                    ))
                    .to_u128_with_decimals(USD_DECIMALS)
                    .unwrap()
        }

        Ok(GetUserBorrowedUsdResponse {
            user_borrowed_usd: Uint128::from(user_borrowed_usd),
        })
    }

    pub fn get_contract_balance_by_token(
        deps: Deps,
        env: Env,
        denom: String,
    ) -> StdResult<Uint128> {
        let contract_address = env.contract.address;
        let coins: Vec<Coin> = deps.querier.query_all_balances(contract_address)?;

        let balance = coins
            .into_iter()
            .find(|coin| coin.denom == denom)
            .map_or(Uint128::zero(), |coin| coin.amount);

        Ok(balance)
    }

    pub fn get_available_to_borrow(
        deps: Deps,
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<Uint128> {
        let mut available_to_borrow = 0u128;
        
        let sum_collateral_balance_usd = get_user_deposited_usd(deps, env.clone(), user.clone())
            .unwrap()
            .user_deposited_usd
            .u128();

        // maximum amount allowed for borrowing
        let max_allowed_borrow_amount_usd = sum_collateral_balance_usd * 8u128 / 10u128;

        let sum_user_borrow_balance_usd = get_user_borrowed_usd(deps, env.clone(), user.clone())
            .unwrap()
            .user_borrowed_usd
            .u128();

        if max_allowed_borrow_amount_usd > sum_user_borrow_balance_usd {
            let token_decimals = get_token_decimal(deps, denom.clone())
                .unwrap()
                .u128() as u32;

            let price = get_price(deps, denom.clone())
                .unwrap()
                .u128();

            available_to_borrow = 
                Decimal::from_i128_with_scale(
                    (max_allowed_borrow_amount_usd
                        - sum_user_borrow_balance_usd) as i128,
                    USD_DECIMALS,
                )
                    .div(Decimal::from_i128_with_scale(
                        price as i128,
                        USD_DECIMALS,
                    ))
                    .to_u128_with_decimals(token_decimals)
                    .unwrap();

            let token_liquidity = get_contract_balance_by_token(deps, env.clone(), denom.clone())
                .unwrap()
                .u128();

            if available_to_borrow > token_liquidity {
                available_to_borrow = token_liquidity
            }
        }

        Ok(Uint128::from(available_to_borrow))
    }

    pub fn get_available_to_redeem(
        deps: Deps,
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<Uint128> {
        let mut available_to_redeem = 0u128;

        let user_token_balance =
            get_deposit(deps, env.clone(), user.clone(), denom.clone())
                .unwrap()
                .balance
                .u128();

        if user_token_balance != 0 {
            let sum_collateral_balance_usd = get_user_deposited_usd(deps, env.clone(), user.clone())
                .unwrap()
                .user_deposited_usd
                .u128();

            let sum_borrow_balance_usd = get_user_borrowed_usd(deps, env.clone(), user.clone())
                .unwrap()
                .user_borrowed_usd
                .u128();

            let required_collateral_balance_usd = sum_borrow_balance_usd * 10u128 / 8u128;

            let token_liquidity = get_contract_balance_by_token(deps, env.clone(), denom.clone())
                .unwrap()
                .u128();

            if sum_collateral_balance_usd >= required_collateral_balance_usd {
                let token_decimals = get_token_decimal(deps, denom.clone())
                    .unwrap()
                    .u128() as u32;

                let price = get_price(deps, denom.clone())
                    .unwrap()
                    .u128();

                available_to_redeem = 
                    Decimal::from_i128_with_scale(
                        (sum_collateral_balance_usd
                            - required_collateral_balance_usd) as i128,
                        USD_DECIMALS,
                    )
                        .div(Decimal::from_i128_with_scale(
                            price as i128,
                            USD_DECIMALS,
                        ))
                        .to_u128_with_decimals(token_decimals)
                        .unwrap();

                if available_to_redeem > user_token_balance {
                    available_to_redeem = user_token_balance
                }

                if available_to_redeem > token_liquidity {
                    available_to_redeem = token_liquidity
                }
            } 
        }

        Ok(Uint128::from(available_to_redeem))
    }

    pub fn get_total_deposited_by_token_usd(
        deps: Deps,
        env: Env,
        denom: String
    ) -> StdResult<Uint128> {
        let users_mm_token_balances_iter: StdResult<Vec<_>> = USER_MM_TOKEN_BALANCE
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        let mut sum_mm_token_balance = 0u128;
        for users_mm_token_balances in users_mm_token_balances_iter.unwrap() {
            if users_mm_token_balances.0 .1 == denom {
                sum_mm_token_balance += users_mm_token_balances.1.u128();
            }
        }

        let mm_token_price = get_mm_token_price(deps.clone(), env.clone(), denom.clone())
            .unwrap()
            .u128();

        let token_decimals = get_token_decimal(deps, denom.clone())
            .unwrap()
            .u128() as u32;

        let sum_token_balance =
            Decimal::from_i128_with_scale(sum_mm_token_balance as i128, token_decimals)
                .mul(Decimal::from_i128_with_scale(
                    mm_token_price as i128,
                    token_decimals,
                ))
                .to_u128_with_decimals(token_decimals)
                .unwrap();

        let price = get_price(deps, denom.clone())
            .unwrap()
            .u128();

        let sum_token_balance_usd = 
            Decimal::from_i128_with_scale(
                sum_token_balance as i128,
                token_decimals,
            )
                .mul(Decimal::from_i128_with_scale(
                    price as i128,
                    USD_DECIMALS,
                ))
                .to_u128_with_decimals(USD_DECIMALS)
                .unwrap();

        Ok(Uint128::from(sum_token_balance_usd))
    }

    pub fn get_total_borrowed_by_token_usd(deps: Deps, denom: String) -> StdResult<Uint128> {
        let total_borrowed_amount = TOTAL_BORROW_DATA
            .load(deps.storage, denom.clone())
            .unwrap()
            .total_borrowed_amount;

        let token_decimals = get_token_decimal(deps, denom.clone())
            .unwrap()
            .u128() as u32;

        let price = get_price(deps, denom.clone())
            .unwrap()
            .u128();

        let total_borrowed_amount_usd = 
            Decimal::from_i128_with_scale(
                total_borrowed_amount as i128,
                token_decimals,
            )
                .mul(Decimal::from_i128_with_scale(
                    price as i128,
                    USD_DECIMALS,
                ))
                .to_u128_with_decimals(USD_DECIMALS)
                .unwrap();

        Ok(Uint128::from(total_borrowed_amount_usd))
    }

    pub fn get_total_reserves_by_token(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let contract_balance = get_contract_balance_by_token(deps, env, denom.clone())
            .unwrap()
            .u128();
        let borrowed_by_token = get_total_borrowed_by_token_usd(deps, denom.clone())
            .unwrap()
            .u128();

        Ok(Uint128::from(contract_balance + borrowed_by_token))
    }

    pub fn get_available_liquidity_by_token(
        deps: Deps,
        env: Env,
        denom: String,
    ) -> StdResult<Uint128> {
        let contract_balance = get_contract_balance_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        Ok(Uint128::from(contract_balance))
    }

    pub fn get_utilization_rate_by_token(
        deps: Deps,
        env: Env,
        denom: String,
    ) -> StdResult<Uint128> {
        let reserves_by_token = get_total_reserves_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        let borrowed_by_token = get_total_borrowed_by_token_usd(deps, denom.clone())
            .unwrap()
            .u128();

        if reserves_by_token != 0 {
            Ok(Uint128::from(
                borrowed_by_token * HUNDRED_PERCENT / reserves_by_token,
            ))
        } else {
            Ok(Uint128::from(0u128))
        }
    }
}
