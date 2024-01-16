use crate::contract::query::{fetch_price_by_token, get_admin, get_all_users_with_borrows, get_available_liquidity_by_token, get_available_to_borrow, get_available_to_redeem, get_current_liquidity_index_ln, get_interest_rate, get_liquidity_index_last_update, get_liquidity_rate, get_mm_token_price, get_pyth_contract, get_pyth_price_feed_ids, get_reserve_configuration, get_supported_tokens, get_token_decimal, get_tokens_interest_rate_model_params, get_total_borrow_data, get_total_borrowed_by_token, get_total_deposited_by_token, get_total_reserves_by_token, get_user_borrow_amount_with_interest, get_user_borrowed_usd, get_user_borrowing_info, get_user_collateral_usd, get_user_deposited_usd, get_user_liquidation_threshold, get_user_max_allowed_borrow_amount_usd, get_user_utilization_rate, get_users_balances, get_utilization_rate_by_token, user_deposit_as_collateral};

use crate::msg::{
    LiquidityIndexData, ReserveConfiguration, TokenInfo, TokenInterestRateModelParams,
    TotalBorrowData, UserBorrowingInfo,
};

use crate::state::{
    IS_TESTING, LIQUIDITY_INDEX_DATA, PRICES, PRICE_FEED_IDS, PRICE_UPDATER_ADDRESS, PYTH_CONTRACT,
    TOTAL_BORROW_DATA, USER_BORROWING_INFO, USER_DEPOSIT_AS_COLLATERAL,
};

use rust_decimal::prelude::{Decimal, MathematicalOps};

use cosmwasm_std::to_json_binary;
use std::ops::{Add, Div, Mul};

use {
    crate::contract::query::get_deposit,
    crate::{
        error::ContractError,
        msg::InstantiateMsg,
        msg::{ExecuteMsg, QueryMsg},
        state::{
            ADMIN, RESERVE_CONFIGURATION, SUPPORTED_TOKENS,
            TOKENS_INTEREST_RATE_MODEL_PARAMS, USER_MM_TOKEN_BALANCE,
        },
    },
    cosmwasm_std::{
        coins, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Timestamp,
        Uint128,
    },
    cw2::set_contract_version,
};

const CONTRACT_NAME: &str = "crates.io:lending";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PERCENT_DECIMALS: u32 = 5;
const HUNDRED_PERCENT: u128 = 100 * 10u128.pow(PERCENT_DECIMALS);

const INTEREST_RATE_DECIMALS: u32 = 18;
const INTEREST_RATE_MULTIPLIER: u128 = 10u128.pow(INTEREST_RATE_DECIMALS);
const HUNDRED: u128 = 100;
const YEAR_IN_SECONDS: u128 = 31536000; // 365 days

const USD_DECIMALS: u32 = 8;

pub trait DecimalExt {
    fn to_u128_with_decimals(&self, decimals: u32) -> StdResult<u128>;
}

impl DecimalExt for Decimal {
    // converting high-precise numbers into u128
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
    // initializing contract with a given parameters
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    IS_TESTING.save(deps.storage, &msg.is_testing)?;
    PRICE_UPDATER_ADDRESS.save(deps.storage, &msg.price_updater_addr)?;
    ADMIN.save(deps.storage, &msg.admin)?;
    PYTH_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(msg.pyth_contract_addr.as_ref())?,
    )?;

    for price_id in msg.price_ids.iter() {
        let price_id = price_id.clone();
        PRICE_FEED_IDS.save(deps.storage, price_id.0.clone(), &price_id.1.clone())?;
    }

    for token in msg.supported_tokens {
        if !msg.is_testing {
            let price = fetch_price_by_token(deps.as_ref(), env.clone(), token.0.clone())
                .unwrap()
                .u128();

            PRICES.save(deps.storage, token.0.clone(), &price)?;
        }

        SUPPORTED_TOKENS.save(
            deps.storage,
            token.0.clone(),
            &TokenInfo {
                denom: token.0.clone(),
                name: token.1,
                symbol: token.2,
                decimals: token.3,
            },
        )?;

        TOTAL_BORROW_DATA.save(
            deps.storage,
            token.0.clone(),
            &TotalBorrowData {
                denom: token.0.clone(),
                total_borrowed_amount: 0u128,
                expected_annual_interest_income: 0u128,
                average_interest_rate: 0u128,
                timestamp: env.block.time,
            },
        )?;

        LIQUIDITY_INDEX_DATA.save(
            deps.storage,
            token.0.clone(),
            &LiquidityIndexData {
                denom: token.0.clone(),
                liquidity_index_ln: 0u128,
                timestamp: env.block.time,
            },
        )?;
    }

    for params in msg.reserve_configuration {
        RESERVE_CONFIGURATION.save(
            deps.storage,
            params.0.clone(),
            &ReserveConfiguration {
                denom: params.0,
                loan_to_value_ratio: params.1,
                liquidation_threshold: params.2,
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
                optimal_utilisation_ratio: params.4,
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
            let amount = amount.u128();

            assert!(amount > 0, "Amount should be a positive number");

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            execute_update_liquidity_index_data(&mut deps, env.clone(), denom.clone())?;

            let current_balance = get_deposit(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                denom.clone(),
            )
                .unwrap()
                .balance
                .u128();

            assert!(
                current_balance >= amount,
                "The account doesn't have enough digital tokens to do withdraw"
            );

            let remaining = current_balance - amount;

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
        ExecuteMsg::AddMarkets {
            denom,
            name,
            symbol,
            decimals,
            loan_to_value_ratio,
            liquidation_threshold,
            min_interest_rate,
            safe_borrow_max_rate,
            rate_growth_factor,
            optimal_utilisation_ratio,
        } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            assert!(
                !SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There already exists such a supported token"
            );

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

            RESERVE_CONFIGURATION.save(
                deps.storage,
                denom.clone(),
                &ReserveConfiguration {
                    denom: denom.clone(),
                    loan_to_value_ratio,
                    liquidation_threshold,
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
                    optimal_utilisation_ratio,
                },
            )?;

            TOTAL_BORROW_DATA.save(
                deps.storage,
                denom.clone(),
                &TotalBorrowData {
                    denom: denom.clone(),
                    total_borrowed_amount: 0u128,
                    expected_annual_interest_income: 0u128,
                    average_interest_rate: 0u128,
                    timestamp: env.block.time,
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
                "The amount to be borrowed is not available"
            );

            assert!(
                get_available_liquidity_by_token(deps.as_ref(), env.clone(), denom.clone())
                    .unwrap()
                    .u128()
                    >= amount.u128()
            );

            execute_update_liquidity_index_data(&mut deps, env.clone(), denom.clone())?;

            let user_borrow_amount_with_interest = get_user_borrow_amount_with_interest(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                denom.clone(),
            )
                .unwrap()
                .u128();

            let user_borrowing_info = get_user_borrowing_info(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string().clone(),
                denom.clone(),
            )
                .unwrap();

            let new_user_borrow_amount: u128 = user_borrow_amount_with_interest + amount.u128();

            let current_interest_rate =
                get_interest_rate(deps.as_ref(), env.clone(), denom.clone())
                    .unwrap()
                    .u128();

            let borrowed_token_decimals = get_token_decimal(deps.as_ref(), denom.clone())
                .unwrap()
                .u128() as u32;

            let average_interest_rate = (Decimal::from_i128_with_scale(
                user_borrow_amount_with_interest as i128,
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

            let total_borrowed_amount = total_borrow_data.total_borrowed_amount
                - user_borrowing_info.borrowed_amount.u128()
                + new_user_borrow_amount;

            let total_average_interest_rate = HUNDRED
                * Decimal::from_i128_with_scale(
                expected_annual_interest_income as i128,
                INTEREST_RATE_DECIMALS,
            )
                .div(Decimal::from_i128_with_scale(
                    total_borrowed_amount as i128,
                    borrowed_token_decimals,
                ))
                .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                .unwrap();

            let new_total_borrow_data = TotalBorrowData {
                denom: denom.clone(),
                total_borrowed_amount: total_borrowed_amount,
                expected_annual_interest_income: expected_annual_interest_income,
                average_interest_rate: total_average_interest_rate,
                timestamp: env.block.time,
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
        ExecuteMsg::UpdatePrice { denom, price } => {
            // if Testing mode, directly set prices for asset
            // if not Testing mode and price argument passed as 0, PRICE_UPDATER_ADDRESS fetching price from Pyth contract, if not available, leaving as is
            if IS_TESTING.load(deps.storage).unwrap() {
                assert_eq!(
                    info.sender.to_string(),
                    ADMIN.load(deps.storage).unwrap(),
                    "This functionality is allowed for admin only"
                );

                assert!(
                    SUPPORTED_TOKENS.has(deps.storage, denom.as_ref().unwrap().clone()),
                    "There is no such supported token yet"
                );

                PRICES.save(deps.storage, denom.unwrap().clone(), &price.unwrap())?;
            } else {
                assert_eq!(
                    info.sender.to_string(),
                    PRICE_UPDATER_ADDRESS.load(deps.storage).unwrap(),
                    "This functionality is allowed for PRICE_UPDATER_ADDRESS only"
                );

                if price.unwrap() == 0 {
                    for token in get_supported_tokens(deps.as_ref())
                        .unwrap()
                        .supported_tokens
                    {
                        let price =
                            fetch_price_by_token(deps.as_ref(), env.clone(), token.denom.clone())
                                .unwrap()
                                .u128();

                        PRICES.save(deps.storage, token.denom, &price)?;
                    }
                }
            }

            Ok(Response::new())
        }
        ExecuteMsg::SetReserveConfiguration {
            denom,
            loan_to_value_ratio,
            liquidation_threshold,
        } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            RESERVE_CONFIGURATION.save(
                deps.storage,
                denom.clone(),
                &ReserveConfiguration {
                    denom: denom.clone(),
                    loan_to_value_ratio,
                    liquidation_threshold,
                },
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::SetTokenInterestRateModelParams {
            denom,
            min_interest_rate,
            safe_borrow_max_rate,
            rate_growth_factor,
            optimal_utilisation_ratio,
        } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            TOKENS_INTEREST_RATE_MODEL_PARAMS.save(
                deps.storage,
                denom.clone(),
                &TokenInterestRateModelParams {
                    denom: denom.clone(),
                    min_interest_rate,
                    safe_borrow_max_rate,
                    rate_growth_factor,
                    optimal_utilisation_ratio,
                },
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::ToggleCollateralSetting { denom } => {
            let use_user_deposit_as_collateral =
                user_deposit_as_collateral(deps.as_ref(), info.sender.to_string(), denom.clone())
                    .unwrap();

            if use_user_deposit_as_collateral {
                let user_token_balance = get_deposit(
                    deps.as_ref(),
                    env.clone(),
                    info.sender.to_string(),
                    denom.clone(),
                )
                    .unwrap()
                    .balance
                    .u128();

                if user_token_balance != 0 {
                    let token_decimals = get_token_decimal(deps.as_ref(), denom.clone())
                        .unwrap()
                        .u128() as u32;

                    let price = fetch_price_by_token(deps.as_ref(), env.clone(), denom.clone())
                        .unwrap()
                        .u128();

                    let user_token_balance_usd =
                        Decimal::from_i128_with_scale(user_token_balance as i128, token_decimals)
                            .mul(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                            .to_u128_with_decimals(USD_DECIMALS)
                            .unwrap();

                    let sum_collateral_balance_usd = get_user_collateral_usd(
                        deps.as_ref(),
                        env.clone(),
                        info.sender.to_string(),
                    )
                        .unwrap()
                        .u128();

                    let sum_borrow_balance_usd =
                        get_user_borrowed_usd(deps.as_ref(), env.clone(), info.sender.to_string())
                            .unwrap()
                            .u128();

                    let user_liquidation_threshold = get_user_liquidation_threshold(
                        deps.as_ref(),
                        env.clone(),
                        info.sender.to_string(),
                    )
                        .unwrap()
                        .u128();

                    assert!(
                        sum_borrow_balance_usd * HUNDRED_PERCENT / user_liquidation_threshold < sum_collateral_balance_usd - user_token_balance_usd,
                        "The collateral has already using to collateralise the borrowing. Not enough available balance"
                    );
                }
            }

            USER_DEPOSIT_AS_COLLATERAL.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &!use_user_deposit_as_collateral,
            )?;

            Ok(Response::new())
        }
        ExecuteMsg::Liquidation { user } => {
            let user_utilization_rate =
                get_user_utilization_rate(deps.as_ref(), env.clone(), user.clone()).unwrap();

            let user_liquidation_threshold =
                get_user_liquidation_threshold(deps.as_ref(), env.clone(), user.clone()).unwrap();

            assert!(
                user_utilization_rate >= user_liquidation_threshold,
                "User borrowing has not reached the threshold of liquidation"
            );

            for token in get_supported_tokens(deps.as_ref())
                .unwrap()
                .supported_tokens
            {
                execute_update_liquidity_index_data(&mut deps, env.clone(), token.denom.clone())?;

                let use_user_deposit_as_collateral =
                    user_deposit_as_collateral(deps.as_ref(), user.clone(), token.denom.clone())
                        .unwrap();

                let mut user_token_balance = 0u128;
                if use_user_deposit_as_collateral {
                    user_token_balance = get_deposit(
                        deps.as_ref(),
                        env.clone(),
                        user.clone(),
                        token.denom.clone(),
                    )
                        .unwrap()
                        .balance
                        .u128();

                    USER_MM_TOKEN_BALANCE.save(
                        deps.storage,
                        (user.clone(), token.denom.clone()),
                        &Uint128::from(0u128),
                    )?;
                }

                let user_borrow_amount_with_interest = get_user_borrow_amount_with_interest(
                    deps.as_ref(),
                    env.clone(),
                    user.clone(),
                    token.denom.clone(),
                )
                    .unwrap()
                    .u128();

                if user_borrow_amount_with_interest > 0 || user_token_balance > 0 {
                    let liquidator_balance = get_deposit(
                        deps.as_ref(),
                        env.clone(),
                        info.sender.to_string(),
                        token.denom.clone(),
                    )
                        .unwrap()
                        .balance
                        .u128();

                    let token_decimals = get_token_decimal(deps.as_ref(), token.denom.clone())
                        .unwrap()
                        .u128() as u32;

                    if user_borrow_amount_with_interest > 0 {
                        assert!(
                            liquidator_balance >= user_borrow_amount_with_interest,
                            "The liquidator does not have enough deposit balance for liquidation"
                        );

                        let user_borrowing_info = get_user_borrowing_info(
                            deps.as_ref(),
                            env.clone(),
                            user.clone(),
                            token.denom.clone(),
                        )
                            .unwrap();

                        let new_user_borrowing_info = UserBorrowingInfo {
                            borrowed_amount: Uint128::from(0u128),
                            average_interest_rate: Uint128::zero(),
                            timestamp: env.block.time,
                        };

                        let total_borrow_data =
                            get_total_borrow_data(deps.as_ref(), token.denom.clone())
                                .unwrap_or_default();

                        let expected_annual_interest_income = total_borrow_data
                            .expected_annual_interest_income
                            - Decimal::from_i128_with_scale(
                            (user_borrowing_info.borrowed_amount.u128()) as i128,
                            token_decimals,
                        )
                            .mul(Decimal::from_i128_with_scale(
                                (user_borrowing_info.average_interest_rate.u128() / HUNDRED)
                                    as i128,
                                INTEREST_RATE_DECIMALS,
                            ))
                            .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                            .unwrap();

                        let total_borrowed_amount = total_borrow_data.total_borrowed_amount
                            - user_borrowing_info.borrowed_amount.u128();

                        let mut total_average_interest_rate = 0u128;
                        if total_borrowed_amount != 0u128 {
                            total_average_interest_rate = HUNDRED
                                * Decimal::from_i128_with_scale(
                                expected_annual_interest_income as i128,
                                INTEREST_RATE_DECIMALS,
                            )
                                .div(Decimal::from_i128_with_scale(
                                    total_borrowed_amount as i128,
                                    token_decimals,
                                ))
                                .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                                .unwrap();
                        }

                        let new_total_borrow_data = TotalBorrowData {
                            denom: token.denom.clone(),
                            total_borrowed_amount: total_borrowed_amount,
                            expected_annual_interest_income: expected_annual_interest_income,
                            average_interest_rate: total_average_interest_rate,
                            timestamp: env.block.time,
                        };

                        USER_BORROWING_INFO.save(
                            deps.storage,
                            (user.clone(), token.denom.clone()),
                            &new_user_borrowing_info,
                        )?;

                        TOTAL_BORROW_DATA.save(
                            deps.storage,
                            token.denom.clone(),
                            &new_total_borrow_data,
                        )?;
                    }

                    let new_liquidator_token_balance =
                        liquidator_balance + user_token_balance - user_borrow_amount_with_interest;

                    let mm_token_price =
                        get_mm_token_price(deps.as_ref(), env.clone(), token.denom.clone())
                            .unwrap()
                            .u128();

                    let new_liquidator_mm_token_balance = Decimal::from_i128_with_scale(
                        new_liquidator_token_balance as i128,
                        token_decimals,
                    )
                        .div(Decimal::from_i128_with_scale(
                            mm_token_price as i128,
                            token_decimals,
                        ))
                        .to_u128_with_decimals(token_decimals)
                        .unwrap();

                    USER_MM_TOKEN_BALANCE.save(
                        deps.storage,
                        (info.sender.to_string(), token.denom.clone()),
                        &Uint128::from(new_liquidator_mm_token_balance),
                    )?;
                }
            }

            Ok(Response::new())
        }
        ExecuteMsg::Repay {} => {
            if info.funds.is_empty() {
                return Err(ContractError::CustomError {
                    val: "Funds not transferred!".to_string(),
                });
            }

            assert_eq!(info.funds.len(), 1, "You have to repay one asset per time");

            let repay_token = info.funds.first().unwrap();
            let mut repay_amount = repay_token.amount.u128();

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, repay_token.denom.clone()),
                "There is no such supported token yet"
            );

            let user_borrowing_info = get_user_borrowing_info(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string().clone(),
                repay_token.denom.clone(),
            )
                .unwrap();

            execute_update_liquidity_index_data(&mut deps, env.clone(), repay_token.denom.clone())?;

            let user_borrow_amount_with_interest = get_user_borrow_amount_with_interest(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                repay_token.denom.clone(),
            )
                .unwrap()
                .u128();

            let mut remaining_amount = 0u128;
            let mut average_interest_rate = user_borrowing_info.average_interest_rate;
            if repay_amount >= user_borrow_amount_with_interest {
                remaining_amount = repay_amount - user_borrow_amount_with_interest;
                repay_amount = user_borrow_amount_with_interest;
                average_interest_rate = Uint128::zero();
            }

            let new_user_borrowing_info = UserBorrowingInfo {
                borrowed_amount: Uint128::from(user_borrow_amount_with_interest - repay_amount),
                average_interest_rate: average_interest_rate,
                timestamp: env.block.time,
            };

            let total_borrow_data =
                get_total_borrow_data(deps.as_ref(), repay_token.denom.clone()).unwrap_or_default();

            let repay_token_decimals = get_token_decimal(deps.as_ref(), repay_token.denom.clone())
                .unwrap()
                .u128() as u32;

            let expected_annual_interest_income = total_borrow_data.expected_annual_interest_income
                + Decimal::from_i128_with_scale(
                (user_borrow_amount_with_interest - user_borrowing_info.borrowed_amount.u128())
                    as i128,
                repay_token_decimals,
            )
                .mul(Decimal::from_i128_with_scale(
                    (user_borrowing_info.average_interest_rate.u128() / HUNDRED) as i128,
                    INTEREST_RATE_DECIMALS,
                ))
                .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                .unwrap()
                - Decimal::from_i128_with_scale((repay_amount) as i128, repay_token_decimals)
                .mul(Decimal::from_i128_with_scale(
                    (user_borrowing_info.average_interest_rate.u128() / HUNDRED) as i128,
                    INTEREST_RATE_DECIMALS,
                ))
                .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                .unwrap();

            let total_borrowed_amount = total_borrow_data.total_borrowed_amount
                + user_borrow_amount_with_interest
                - user_borrowing_info.borrowed_amount.u128()
                - repay_amount;

            let mut total_average_interest_rate = 0u128;
            if total_borrowed_amount != 0u128 {
                total_average_interest_rate = HUNDRED
                    * Decimal::from_i128_with_scale(
                    expected_annual_interest_income as i128,
                    INTEREST_RATE_DECIMALS,
                )
                    .div(Decimal::from_i128_with_scale(
                        total_borrowed_amount as i128,
                        repay_token_decimals,
                    ))
                    .to_u128_with_decimals(INTEREST_RATE_DECIMALS)
                    .unwrap();
            }

            let new_total_borrow_data = TotalBorrowData {
                denom: repay_token.denom.clone(),
                total_borrowed_amount: total_borrowed_amount,
                expected_annual_interest_income: expected_annual_interest_income,
                average_interest_rate: total_average_interest_rate,
                timestamp: env.block.time,
            };

            USER_BORROWING_INFO.save(
                deps.storage,
                (info.sender.to_string(), repay_token.denom.clone()),
                &new_user_borrowing_info,
            )?;

            TOTAL_BORROW_DATA.save(
                deps.storage,
                repay_token.denom.clone(),
                &new_total_borrow_data,
            )?;

            if remaining_amount > 0 {
                Ok(Response::new().add_message(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: coins(remaining_amount, repay_token.denom.clone()),
                }))
            } else {
                Ok(Response::default())
            }
        }
        ExecuteMsg::UpdatePythContract { pyth_contract_addr } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            PYTH_CONTRACT.save(
                deps.storage,
                &deps.api.addr_validate(pyth_contract_addr.as_ref())?,
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::AddPriceFeedIds { price_ids } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            assert!(!price_ids.is_empty(), "Couldn't pass empty parameters");

            for price_id in price_ids.iter() {
                let price_id = price_id.clone();
                PRICE_FEED_IDS.save(deps.storage, price_id.0.clone(), &price_id.1.clone())?;
            }

            Ok(Response::default())
        }
        ExecuteMsg::UpdatePriceUpdaterAddr { price_updater_addr } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            PRICE_UPDATER_ADDRESS.save(
                deps.storage,
                &price_updater_addr,
            )?;

            Ok(Response::default())
        }
        ExecuteMsg::UpdateAdmin { admin } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            ADMIN.save(
                deps.storage,
                &admin,
            )?;

            Ok(Response::default())
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
            to_json_binary(&get_deposit(deps, env, address, denom)?)
        }
        QueryMsg::UserDepositAsCollateral { address, denom } => {
            to_json_binary(&user_deposit_as_collateral(deps, address, denom)?)
        }
        QueryMsg::GetPrice { denom } => to_json_binary(&fetch_price_by_token(deps, env, denom)?),
        QueryMsg::GetUserBorrowAmountWithInterest { address, denom } => to_json_binary(
            &query::get_user_borrow_amount_with_interest(deps, env, address, denom)?,
        ),
        QueryMsg::GetUserBorrowingInfo { address, denom } => {
            to_json_binary(&query::get_user_borrowing_info(deps, env, address, denom)?)
        }
        QueryMsg::GetTotalBorrowData { denom } => {
            to_json_binary(&query::get_total_borrow_data(deps, denom)?)
        }
        QueryMsg::GetSupportedTokens {} => to_json_binary(&get_supported_tokens(deps)?),
        QueryMsg::GetReserveConfiguration {} => to_json_binary(&get_reserve_configuration(deps)?),
        QueryMsg::GetTokensInterestRateModelParams {} => {
            to_json_binary(&get_tokens_interest_rate_model_params(deps)?)
        }
        QueryMsg::GetInterestRate { denom } => {
            to_json_binary(&get_interest_rate(deps, env, denom)?)
        }
        QueryMsg::GetLiquidityRate { denom } => {
            to_json_binary(&get_liquidity_rate(deps, env, denom)?)
        }
        QueryMsg::GetCurrentLiquidityIndexLn { denom } => {
            to_json_binary(&get_current_liquidity_index_ln(deps, env, denom)?)
        }
        QueryMsg::GetMmTokenPrice { denom } => {
            to_json_binary(&get_mm_token_price(deps, env, denom)?)
        }
        QueryMsg::GetUserDepositedUsd { address } => {
            to_json_binary(&get_user_deposited_usd(deps, env, address)?)
        }
        QueryMsg::GetUserCollateralUsd { address } => {
            to_json_binary(&get_user_collateral_usd(deps, env, address)?)
        }
        QueryMsg::GetUserBorrowedUsd { address } => {
            to_json_binary(&get_user_borrowed_usd(deps, env, address)?)
        }
        QueryMsg::GetUserUtilizationRate { address } => {
            to_json_binary(&get_user_utilization_rate(deps, env, address)?)
        }
        QueryMsg::GetUserLiquidationThreshold { address } => {
            to_json_binary(&get_user_liquidation_threshold(deps, env, address)?)
        }
        QueryMsg::GetAvailableToBorrow { address, denom } => {
            to_json_binary(&get_available_to_borrow(deps, env, address, denom)?)
        }
        QueryMsg::GetAvailableToRedeem { address, denom } => {
            to_json_binary(&get_available_to_redeem(deps, env, address, denom)?)
        }
        QueryMsg::GetTotalReservesByToken { denom } => {
            to_json_binary(&get_total_reserves_by_token(deps, env, denom)?)
        }
        QueryMsg::GetTotalDepositedByToken { denom } => {
            to_json_binary(&get_total_deposited_by_token(deps, env, denom)?)
        }
        QueryMsg::GetTotalBorrowedByToken { denom } => {
            to_json_binary(&get_total_borrowed_by_token(deps, env, denom)?)
        }
        QueryMsg::GetAvailableLiquidityByToken { denom } => {
            to_json_binary(&get_available_liquidity_by_token(deps, env, denom)?)
        }
        QueryMsg::GetUtilizationRateByToken { denom } => {
            to_json_binary(&get_utilization_rate_by_token(deps, env, denom)?)
        }
        QueryMsg::GetLiquidityIndexLastUpdate { denom } => {
            to_json_binary(&get_liquidity_index_last_update(deps, denom)?)
        }
        QueryMsg::GetUserMaxAllowedBorrowAmountUsd { address } => {
            to_json_binary(&get_user_max_allowed_borrow_amount_usd(deps, env, address)?)
        }
        QueryMsg::GetAllUsersWithBorrows {} => {
            to_json_binary(&get_all_users_with_borrows(deps, env)?)
        }
        QueryMsg::GetPythContract {} => to_json_binary(&get_pyth_contract(deps)?),
        QueryMsg::GetPriceFeedIds {} => to_json_binary(&get_pyth_price_feed_ids(deps)?),
        QueryMsg::GetAdmin {} => to_json_binary(&get_admin(deps)?),
        QueryMsg::GetUserBalances { address } => to_json_binary(&get_users_balances(deps, env, address)?),
    }
}

pub mod query {
    use super::*;
    use std::ops::Mul;

    use crate::msg::{GetBalanceResponse, GetReserveConfigurationResponse, GetSupportedTokensResponse, GetTokensInterestRateModelParamsResponse, TotalBorrowData, UserBorrowingInfo, UserDataByToken};
    use cosmwasm_std::Order::Ascending;
    use cosmwasm_std::{Coin, Order};
    use pyth_sdk_cw::{query_price_feed, PriceFeedResponse, PriceIdentifier};

    pub fn get_deposit(
        deps: Deps,
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<GetBalanceResponse> {
        // calculates user deposit including deposit interest
        let token_decimals = get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

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

    pub fn user_deposit_as_collateral(deps: Deps, user: String, denom: String) -> StdResult<bool> {
        let use_user_deposit_as_collateral = USER_DEPOSIT_AS_COLLATERAL
            .load(deps.storage, (user, denom.clone()))
            .unwrap_or_default();

        Ok(use_user_deposit_as_collateral)
    }

    pub fn get_pyth_contract(deps: Deps) -> StdResult<String> {
        Ok(PYTH_CONTRACT.load(deps.storage)?.to_string())
    }

    pub fn get_admin(deps: Deps) -> StdResult<String> {
        Ok(ADMIN.load(deps.storage)?.to_string())
    }

    pub fn get_pyth_price_feed_ids(deps: Deps) -> StdResult<Vec<(String, PriceIdentifier)>> {
        Ok(PRICE_FEED_IDS
            .keys(deps.storage, None, None, Ascending)
            .map(|denom| {
                let token_denom = denom.unwrap();

                let price_identifier = PRICE_FEED_IDS
                    .load(deps.storage, token_denom.clone())
                    .unwrap();

                (token_denom, price_identifier)
            })
            .collect())
    }

    pub fn calc_borrow_amount_with_interest(
        borrowed_amount: u128,
        interest_rate: u128,
        interval: u128,
        token_decimals: u32,
    ) -> StdResult<Uint128> {
        let base = Decimal::from_i128_with_scale(
            (interest_rate / HUNDRED + INTEREST_RATE_MULTIPLIER) as i128,
            INTEREST_RATE_DECIMALS,
        );

        let exponent = Decimal::from_i128_with_scale(
            (interval * INTEREST_RATE_MULTIPLIER / YEAR_IN_SECONDS) as i128,
            INTEREST_RATE_DECIMALS,
        );

        let borrow_amount_with_interest =
            Decimal::from_i128_with_scale(borrowed_amount as i128, token_decimals)
                .mul(base.powd(exponent))
                .to_u128_with_decimals(token_decimals)
                .unwrap();

        Ok(Uint128::from(borrow_amount_with_interest))
    }

    pub fn get_user_borrow_amount_with_interest(
        deps: Deps,
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<Uint128> {
        let current_borrowing_info =
            get_user_borrowing_info(deps.clone(), env.clone(), user.clone(), denom.clone())
                .unwrap();

        let token_decimals = get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

        let borrow_amount_with_interest = calc_borrow_amount_with_interest(
            current_borrowing_info.borrowed_amount.u128(),
            current_borrowing_info.average_interest_rate.u128(),
            (env.block.time.seconds() - current_borrowing_info.timestamp.seconds()) as u128,
            token_decimals,
        )
            .unwrap()
            .u128();

        Ok(Uint128::from(borrow_amount_with_interest))
    }

    pub fn get_liquidity_rate(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let expected_annual_interest_income = TOTAL_BORROW_DATA
            .load(deps.storage, denom.clone())
            .unwrap()
            .expected_annual_interest_income;

        let reserves_by_token = get_total_reserves_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        let token_decimals = get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

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

    pub fn get_mm_token_price(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        // number of tokens that correspond to one mmToken
        let token_decimals = get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

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

    pub fn fetch_price_by_token(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        // if testing mode pulling price from a contract, otherwise fetching from pyth contract
        if IS_TESTING.load(deps.storage)? {
            Ok(Uint128::from(
                PRICES.load(deps.storage, denom).unwrap_or(0u128),
            ))
        } else {
            let pyth_contract = PYTH_CONTRACT.load(deps.storage)?;

            let price_identifier = PRICE_FEED_IDS.load(deps.storage, denom.clone())?;

            let price_feed_response: PriceFeedResponse =
                query_price_feed(&deps.querier, pyth_contract, price_identifier)?;
            let price_feed = price_feed_response.price_feed;

            let mut current_price =
                Uint128::from(PRICES.load(deps.storage, denom).unwrap_or(0u128));

            // if Pyth price is available getting most recent price if not - just load from a contract
            let pyth_current_price =
                price_feed.get_price_no_older_than(env.block.time.seconds() as i64, 60);

            if pyth_current_price.is_some() {
                current_price = Uint128::from(pyth_current_price.unwrap().price as u128)
            }

            Ok(current_price)
        }
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

    pub fn get_reserve_configuration(deps: Deps) -> StdResult<GetReserveConfigurationResponse> {
        let mut result: Vec<ReserveConfiguration> = vec![];

        let all: StdResult<Vec<_>> = RESERVE_CONFIGURATION
            .range(deps.storage, None, None, Order::Ascending)
            .collect();
        for el in all.unwrap() {
            result.push(el.1)
        }

        Ok(GetReserveConfigurationResponse {
            reserve_configuration: result,
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
        let optimal_utilisation_ratio = TOKENS_INTEREST_RATE_MODEL_PARAMS
            .load(deps.storage, denom.clone())
            .unwrap()
            .optimal_utilisation_ratio;

        if utilization_rate <= optimal_utilisation_ratio {
            Ok(Uint128::from(
                min_interest_rate
                    + utilization_rate * (safe_borrow_max_rate - min_interest_rate)
                    / optimal_utilisation_ratio,
            ))
        } else {
            Ok(Uint128::from(
                safe_borrow_max_rate
                    + rate_growth_factor * (utilization_rate - optimal_utilisation_ratio)
                    / (HUNDRED_PERCENT - optimal_utilisation_ratio),
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
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<UserBorrowingInfo> {
        let user_borrowing_info = USER_BORROWING_INFO
            .load(deps.storage, (user, denom.clone()))
            .unwrap_or_default();

        let mut average_interest_rate: Uint128 = user_borrowing_info.average_interest_rate;
        let mut timestamp: Timestamp = user_borrowing_info.timestamp;
        if user_borrowing_info.borrowed_amount == Uint128::zero() {
            let current_interest_rate = get_interest_rate(deps, env.clone(), denom.clone())
                .unwrap()
                .u128();

            average_interest_rate = Uint128::from(current_interest_rate);
            timestamp = env.block.time;
        }

        Ok(UserBorrowingInfo {
            borrowed_amount: user_borrowing_info.borrowed_amount,
            average_interest_rate: average_interest_rate,
            timestamp: timestamp,
        })
    }

    pub fn get_total_borrow_data(deps: Deps, denom: String) -> StdResult<TotalBorrowData> {
        Ok(TOTAL_BORROW_DATA
            .load(deps.storage, denom)
            .unwrap_or_default())
    }

    pub fn get_user_deposited_usd(deps: Deps, env: Env, user: String) -> StdResult<Uint128> {
        let mut user_deposited_usd = 0u128;

        for token in get_supported_tokens(deps).unwrap().supported_tokens {
            let user_deposit = get_deposit(deps, env.clone(), user.clone(), token.denom.clone())
                .unwrap()
                .balance
                .u128();

            let token_decimals =
                get_token_decimal(deps, token.denom.clone()).unwrap().u128() as u32;

            let price = fetch_price_by_token(deps, env.clone(), token.denom.clone())
                .unwrap()
                .u128();

            user_deposited_usd +=
                Decimal::from_i128_with_scale(user_deposit as i128, token_decimals)
                    .mul(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                    .to_u128_with_decimals(USD_DECIMALS)
                    .unwrap()
        }

        Ok(Uint128::from(user_deposited_usd))
    }

    pub fn get_user_collateral_usd(deps: Deps, env: Env, user: String) -> StdResult<Uint128> {
        let mut user_collateral_usd = 0u128;

        for token in get_supported_tokens(deps).unwrap().supported_tokens {
            let use_user_deposit_as_collateral =
                user_deposit_as_collateral(deps, user.clone(), token.denom.clone()).unwrap();

            if use_user_deposit_as_collateral {
                let user_deposit =
                    get_deposit(deps, env.clone(), user.clone(), token.denom.clone())
                        .unwrap()
                        .balance
                        .u128();

                let token_decimals =
                    get_token_decimal(deps, token.denom.clone()).unwrap().u128() as u32;

                let price = fetch_price_by_token(deps, env.clone(), token.denom.clone())
                    .unwrap()
                    .u128();

                user_collateral_usd +=
                    Decimal::from_i128_with_scale(user_deposit as i128, token_decimals)
                        .mul(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                        .to_u128_with_decimals(USD_DECIMALS)
                        .unwrap()
            }
        }

        Ok(Uint128::from(user_collateral_usd))
    }

    pub fn get_user_borrowed_usd(deps: Deps, env: Env, user: String) -> StdResult<Uint128> {
        let mut user_borrowed_usd = 0u128;
        for token in get_supported_tokens(deps).unwrap().supported_tokens {
            let user_borrow_amount_with_interest = get_user_borrow_amount_with_interest(
                deps,
                env.clone(),
                user.clone(),
                token.denom.clone(),
            )
                .unwrap()
                .u128();

            let token_decimals =
                get_token_decimal(deps, token.denom.clone()).unwrap().u128() as u32;

            let price = fetch_price_by_token(deps, env.clone(), token.denom.clone())
                .unwrap()
                .u128();

            user_borrowed_usd += Decimal::from_i128_with_scale(
                user_borrow_amount_with_interest as i128,
                token_decimals,
            )
                .mul(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                .to_u128_with_decimals(USD_DECIMALS)
                .unwrap()
        }

        Ok(Uint128::from(user_borrowed_usd))
    }

    pub fn get_available_liquidity_by_token(
        deps: Deps,
        env: Env,
        denom: String,
    ) -> StdResult<Uint128> {
        let contract_address = env.contract.address;
        let coins: Vec<Coin> = deps.querier.query_all_balances(contract_address)?;

        let liquidity = coins
            .into_iter()
            .find(|coin| coin.denom == denom)
            .map_or(Uint128::zero(), |coin| coin.amount);

        Ok(liquidity)
    }

    pub fn get_user_liquidation_threshold(
        deps: Deps,
        env: Env,
        user: String,
    ) -> StdResult<Uint128> {
        // the minimum borrowing amount in USD, upon reaching which the user's loan positions are liquidated
        let mut liquidation_threshold_borrow_amount_usd = 0u128;
        let mut user_collateral_usd = 0u128;

        for token in get_supported_tokens(deps).unwrap().supported_tokens {
            let use_user_deposit_as_collateral =
                user_deposit_as_collateral(deps, user.clone(), token.denom.clone()).unwrap();

            if use_user_deposit_as_collateral {
                let user_deposit =
                    get_deposit(deps, env.clone(), user.clone(), token.denom.clone())
                        .unwrap()
                        .balance
                        .u128();

                let liquidation_threshold = RESERVE_CONFIGURATION
                    .load(deps.storage, token.denom.clone())
                    .unwrap()
                    .liquidation_threshold;

                let token_decimals =
                    get_token_decimal(deps, token.denom.clone()).unwrap().u128() as u32;

                let price = fetch_price_by_token(deps, env.clone(), token.denom.clone())
                    .unwrap()
                    .u128();

                let user_deposit_usd =
                    Decimal::from_i128_with_scale(user_deposit as i128, token_decimals)
                        .mul(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                        .to_u128_with_decimals(USD_DECIMALS)
                        .unwrap();

                liquidation_threshold_borrow_amount_usd +=
                    user_deposit_usd * liquidation_threshold / HUNDRED_PERCENT;
                user_collateral_usd += user_deposit_usd;
            }
        }

        Ok(Uint128::from(
            liquidation_threshold_borrow_amount_usd * HUNDRED_PERCENT / user_collateral_usd,
        ))
    }

    pub fn get_user_max_allowed_borrow_amount_usd(
        deps: Deps,
        env: Env,
        user: String,
    ) -> StdResult<Uint128> {
        // the maximum amount in USD that a user can borrow
        let mut max_allowed_borrow_amount_usd = 0u128;

        for token in get_supported_tokens(deps).unwrap().supported_tokens {
            let use_user_deposit_as_collateral =
                user_deposit_as_collateral(deps, user.clone(), token.denom.clone()).unwrap();

            if use_user_deposit_as_collateral {
                let user_deposit =
                    get_deposit(deps, env.clone(), user.clone(), token.denom.clone())
                        .unwrap()
                        .balance
                        .u128();

                let loan_to_value_ratio = RESERVE_CONFIGURATION
                    .load(deps.storage, token.denom.clone())
                    .unwrap()
                    .loan_to_value_ratio;

                let token_decimals =
                    get_token_decimal(deps, token.denom.clone()).unwrap().u128() as u32;

                let price = fetch_price_by_token(deps, env.clone(), token.denom.clone())
                    .unwrap()
                    .u128();

                let user_deposit_usd =
                    Decimal::from_i128_with_scale(user_deposit as i128, token_decimals)
                        .mul(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                        .to_u128_with_decimals(USD_DECIMALS)
                        .unwrap();

                max_allowed_borrow_amount_usd +=
                    user_deposit_usd * loan_to_value_ratio / HUNDRED_PERCENT;
            }
        }

        Ok(Uint128::from(max_allowed_borrow_amount_usd))
    }

    pub fn get_available_to_borrow(
        deps: Deps,
        env: Env,
        user: String,
        denom: String,
    ) -> StdResult<Uint128> {
        let mut available_to_borrow = 0u128;

        // maximum amount allowed for borrowing
        let max_allowed_borrow_amount_usd =
            get_user_max_allowed_borrow_amount_usd(deps, env.clone(), user.clone())
                .unwrap()
                .u128();

        let sum_user_borrow_balance_usd = get_user_borrowed_usd(deps, env.clone(), user.clone())
            .unwrap()
            .u128();

        if max_allowed_borrow_amount_usd > sum_user_borrow_balance_usd {
            let token_decimals = get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

            let price = fetch_price_by_token(deps, env.clone(), denom.clone())
                .unwrap()
                .u128();

            available_to_borrow = Decimal::from_i128_with_scale(
                (max_allowed_borrow_amount_usd - sum_user_borrow_balance_usd) as i128,
                USD_DECIMALS,
            )
                .div(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                .to_u128_with_decimals(token_decimals)
                .unwrap();

            let token_liquidity =
                get_available_liquidity_by_token(deps, env.clone(), denom.clone())
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

        let user_token_balance = get_deposit(deps, env.clone(), user.clone(), denom.clone())
            .unwrap()
            .balance
            .u128();

        if user_deposit_as_collateral(deps, user.clone(), denom.clone()).unwrap() {
            if user_token_balance != 0 {
                let sum_collateral_balance_usd =
                    get_user_collateral_usd(deps, env.clone(), user.clone())
                        .unwrap()
                        .u128();

                let sum_borrow_balance_usd = get_user_borrowed_usd(deps, env.clone(), user.clone())
                    .unwrap()
                    .u128();

                let user_liquidation_threshold =
                    get_user_liquidation_threshold(deps, env.clone(), user.clone())
                        .unwrap()
                        .u128();

                let required_collateral_balance_usd =
                    sum_borrow_balance_usd * HUNDRED_PERCENT / user_liquidation_threshold;

                let token_liquidity =
                    get_available_liquidity_by_token(deps, env.clone(), denom.clone())
                        .unwrap()
                        .u128();

                if sum_collateral_balance_usd >= required_collateral_balance_usd {
                    let token_decimals =
                        get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

                    let price = fetch_price_by_token(deps, env.clone(), denom.clone())
                        .unwrap()
                        .u128();

                    available_to_redeem = Decimal::from_i128_with_scale(
                        (sum_collateral_balance_usd - required_collateral_balance_usd) as i128,
                        USD_DECIMALS,
                    )
                        .div(Decimal::from_i128_with_scale(price as i128, USD_DECIMALS))
                        .to_u128_with_decimals(token_decimals)
                        .unwrap();

                    if available_to_redeem > user_token_balance {
                        available_to_redeem = user_token_balance;
                    }

                    if available_to_redeem > token_liquidity {
                        available_to_redeem = token_liquidity;
                    }
                }
            }
        } else {
            available_to_redeem = user_token_balance;
        }

        Ok(Uint128::from(available_to_redeem))
    }

    pub fn get_total_deposited_by_token(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let users_mm_token_balances_iter: StdResult<Vec<_>> = USER_MM_TOKEN_BALANCE
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        let mut sum_mm_token_balance = 0u128;
        for users_mm_token_balances in users_mm_token_balances_iter.unwrap() {
            if users_mm_token_balances.0.1 == denom {
                sum_mm_token_balance += users_mm_token_balances.1.u128();
            }
        }

        let mm_token_price = get_mm_token_price(deps.clone(), env.clone(), denom.clone())
            .unwrap()
            .u128();

        let token_decimals = get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

        let sum_token_balance =
            Decimal::from_i128_with_scale(sum_mm_token_balance as i128, token_decimals)
                .mul(Decimal::from_i128_with_scale(
                    mm_token_price as i128,
                    token_decimals,
                ))
                .to_u128_with_decimals(token_decimals)
                .unwrap();

        Ok(Uint128::from(sum_token_balance))
    }

    pub fn get_total_borrowed_by_token(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let total_borrow_data = get_total_borrow_data(deps, denom.clone()).unwrap_or_default();

        let token_decimals = get_token_decimal(deps, denom.clone()).unwrap().u128() as u32;

        let total_borrowed_amount_with_interest = calc_borrow_amount_with_interest(
            total_borrow_data.total_borrowed_amount,
            total_borrow_data.average_interest_rate,
            (env.block.time.seconds() - total_borrow_data.timestamp.seconds()) as u128,
            token_decimals,
        )
            .unwrap()
            .u128();

        Ok(Uint128::from(total_borrowed_amount_with_interest))
    }

    pub fn get_total_reserves_by_token(deps: Deps, env: Env, denom: String) -> StdResult<Uint128> {
        let token_liquidity = get_available_liquidity_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();
        let borrowed_by_token = get_total_borrowed_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        Ok(Uint128::from(token_liquidity + borrowed_by_token))
    }

    pub fn get_utilization_rate_by_token(
        deps: Deps,
        env: Env,
        denom: String,
    ) -> StdResult<Uint128> {
        let reserves_by_token = get_total_reserves_by_token(deps, env.clone(), denom.clone())
            .unwrap()
            .u128();

        if reserves_by_token != 0 {
            let borrowed_by_token = get_total_borrowed_by_token(deps, env, denom.clone())
                .unwrap()
                .u128();

            Ok(Uint128::from(
                borrowed_by_token * HUNDRED_PERCENT / reserves_by_token,
            ))
        } else {
            Ok(Uint128::from(0u128))
        }
    }

    pub fn get_user_utilization_rate(deps: Deps, env: Env, user: String) -> StdResult<Uint128> {
        let sum_collateral_balance_usd = get_user_collateral_usd(deps, env.clone(), user.clone())
            .unwrap()
            .u128();

        if sum_collateral_balance_usd != 0 {
            let sum_user_borrow_balance_usd =
                get_user_borrowed_usd(deps, env.clone(), user.clone())
                    .unwrap()
                    .u128();

            Ok(Uint128::from(
                sum_user_borrow_balance_usd * HUNDRED_PERCENT / sum_collateral_balance_usd,
            ))
        } else {
            Ok(Uint128::from(0u128))
        }
    }

    pub fn get_all_users_with_borrows(deps: Deps, env: Env) -> StdResult<Vec<String>> {
        let user_borrowed_data: StdResult<Vec<_>> = USER_BORROWING_INFO
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        let mut uniq = user_borrowed_data
            .unwrap()
            .into_iter()
            .map(|((user, _), _)| user)
            .collect::<Vec<String>>();
        uniq.sort();
        uniq.dedup();

        Ok(uniq)
    }

    pub fn get_users_balances(deps: Deps, env: Env, address: String) -> StdResult<Vec<(String, UserDataByToken)>> {
        let mut result = vec![];

        for token in get_supported_tokens(deps)
            .unwrap()
            .supported_tokens {
            let user_deposit = get_deposit(deps, env.clone(), address.clone(), token.denom.clone())
                .unwrap()
                .balance;

            let user_borrow_amount_with_interest = get_user_borrow_amount_with_interest(
                deps,
                env.clone(),
                address.clone(),
                token.denom.clone(),
            )
                .unwrap();
            let user_data_by_token = UserDataByToken {
                deposited: user_deposit,
                borrowed: user_borrow_amount_with_interest,
            };

            result.push((token.denom, user_data_by_token))
        }

        Ok(result)
    }
}
