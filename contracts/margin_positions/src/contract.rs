use crate::contract::query::{fetch_price_by_token, get_collateral_vault_contract, get_deposit, get_lending_contract, get_margin_positions_count, get_order_by_id, get_orders_by_user};
use crate::state::{
    MarginPositionsCount, ADMIN, COLLATERAL_VAULT, IS_TESTING, LENDING_CONTRACT, MARGIN_POSITIONS,
    MARGIN_POSITIONS_COUNT, PRICES, PRICE_FEED_IDS, PRICE_UPDATER_CONTRACT, PYTH_CONTRACT,
    SUPPORTED_TOKENS, USER_DEPOSITED_BALANCE,
};
use crate::utils::TokenInfo;
use cosmwasm_std::{CosmosMsg, WasmMsg};

use crate::utils::{OrderInfo, OrderStatus};
use {
    crate::{
        error::ContractError,
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    },
    cosmwasm_std::{
        coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
        Uint128,
    },
    cw2::set_contract_version,
};
use crate::utils::ExecuteExternal::{Borrow, RedeemFromVaultContractMargin};

const COLLATERAL_VAULT_CONTRACT: &str = "crates.io:collateral_vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const LEVERAGE_DECIMALS: u128 = 2;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, COLLATERAL_VAULT_CONTRACT, CONTRACT_VERSION)?;

    IS_TESTING.save(deps.storage, &msg.is_testing)?;

    PRICE_UPDATER_CONTRACT.save(deps.storage, &msg.price_updater_contract_addr)?;

    ADMIN.save(deps.storage, &msg.admin)?;

    PYTH_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(msg.pyth_contract_addr.as_ref())?,
    )?;
    COLLATERAL_VAULT.save(deps.storage, &msg.collateral_vault_contract)?;

    LENDING_CONTRACT.save(deps.storage, &msg.lending_contract)?;

    MARGIN_POSITIONS_COUNT.save(deps.storage, &MarginPositionsCount::default())?;

    for price_id in msg.price_ids.iter() {
        let price_id = price_id.clone();
        PRICE_FEED_IDS.save(deps.storage, price_id.0.clone(), &price_id.1.clone())?;
    }

    for token in msg.supported_tokens {
        // saving price whilst initialisation in case when Pyth price is not available so that we can get price from contract
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
    }

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
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

            // execute_update_liquidity_index_data(
            //     &mut deps,
            //     env.clone(),
            //     deposited_token.denom.clone(),
            // )?;

            let user_current_token_balance = USER_DEPOSITED_BALANCE
                .load(
                    deps.storage,
                    (info.sender.to_string(), deposited_token.denom.clone()),
                )
                .unwrap_or_else(|_| Uint128::zero());

            let new_user_token_balance = user_current_token_balance.u128() + deposited_token_amount;

            USER_DEPOSITED_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), deposited_token.denom.clone()),
                &Uint128::from(new_user_token_balance),
            )?;

            // Sending funds to the trusted Collateral Vault Contract
            Ok(Response::new().add_message(BankMsg::Send {
                to_address: get_collateral_vault_contract(deps.as_ref()).unwrap(),
                amount: coins(deposited_token.amount.u128(), deposited_token.denom.clone()),
            }))
        }
        ExecuteMsg::Redeem { denom, amount } => {
            let amount = amount.u128();

            assert!(amount > 0, "Amount should be a positive number");

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, denom.clone()),
                "There is no such supported token yet"
            );

            // execute_update_liquidity_index_data(&mut deps, env.clone(), denom.clone())?;

            let current_balance = get_deposit(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                denom.clone(),
            )
                .unwrap()
                .u128();

            assert!(
                current_balance >= amount,
                "The account doesn't have enough digital tokens to do withdraw"
            );

            let remaining = current_balance - amount;

            USER_DEPOSITED_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &Uint128::from(remaining),
            )?;

            // initiating redeem from lending contract if all checks are passed
            let msg_redeem_from_vault_contract_margin = RedeemFromVaultContractMargin {
                denom: denom.clone(),
                amount: Uint128::from(amount),
                user: info.sender.to_string(),
            };

            Ok(
                Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: get_collateral_vault_contract(deps.as_ref()).unwrap(),
                    msg: to_binary(&msg_redeem_from_vault_contract_margin)?,
                    funds: vec![],
                })),
            )
        }
        ExecuteMsg::UpdatePrice { denom, price } => {
            // if not testing mode, PRICE_UPDATER_CONTRACT (backend service) is allowed to update price manually
            if !IS_TESTING.load(deps.storage).unwrap() {
                assert_eq!(
                    info.sender.to_string(),
                    PRICE_UPDATER_CONTRACT.load(deps.storage).unwrap(),
                    "This functionality is allowed for PRICE_UPDATER_CONTRACT only"
                );

                assert!(
                    SUPPORTED_TOKENS.has(deps.storage, denom.as_ref().unwrap().clone()),
                    "There is no such supported token yet"
                );

                PRICES.save(deps.storage, denom.unwrap().clone(), &price.unwrap())?;
            } else {
                assert_eq!(
                    info.sender.to_string(),
                    ADMIN.load(deps.storage).unwrap(),
                    "This functionality is allowed for admin only"
                );

                assert!(
                    SUPPORTED_TOKENS.has(deps.storage, denom.as_ref().unwrap().clone()),
                    "There is no such supported token yet"
                );

                PRICES.save(deps.storage, denom.unwrap(), &price.unwrap())?;
            }

            Ok(Response::new())
        }
        ExecuteMsg::SetCollateralVaultContract { contract } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            COLLATERAL_VAULT.save(deps.storage, &contract)?;
            Ok(Response::default())
        }
        ExecuteMsg::SetLendingContract { contract } => {
            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            LENDING_CONTRACT.save(deps.storage, &contract)?;
            Ok(Response::default())
        }
        ExecuteMsg::CreateOrder {
            order_type,
            amount,
            sell_token_denom,
            leverage,
        } => {
            assert!(
                leverage >= 1 * 10u128.pow(LEVERAGE_DECIMALS as u32),
                "Leverage should be not less than 1.0"
            );

            assert!(
                SUPPORTED_TOKENS.has(deps.storage, sell_token_denom.clone()),
                "There is no such supported token for positions"
            );

            let current_deposit = get_deposit(
                deps.as_ref(),
                env.clone(),
                info.sender.to_string(),
                sell_token_denom.clone(),
            )
                .unwrap()
                .u128();

            assert!(
                current_deposit >= amount.u128(),
                "The account doesn't have enough deposited tokens to do open position"
            );

            let order = OrderInfo {
                order_status: OrderStatus::Pending,
                order_type,
                amount: Uint128::from(amount),
                sell_token_denom: sell_token_denom.clone(),
                leverage,
            };

            let mut margin_positions_count_old = get_margin_positions_count(deps.as_ref()).unwrap();
            MARGIN_POSITIONS_COUNT.save(
                deps.storage,
                &(margin_positions_count_old.increase_count_by_one()),
            )?;

            let margin_positions_count_new = get_margin_positions_count(deps.as_ref()).unwrap();

            MARGIN_POSITIONS.save(
                deps.storage,
                (info.sender.to_string(), margin_positions_count_new.count),
                &order,
            )?;

            let remaining = current_deposit - amount.u128();

            USER_DEPOSITED_BALANCE.save(
                deps.storage,
                (info.sender.to_string(), sell_token_denom.clone()),
                &Uint128::from(remaining),
            )?;
            if leverage != 1 * 10u128.pow(LEVERAGE_DECIMALS as u32) {
                let amount_to_borrow =
                    order.amount.u128() * order.leverage / 10u128.pow(LEVERAGE_DECIMALS as u32);

                let msg_borrow_uncollateralazied = Borrow {
                    denom: sell_token_denom,
                    amount: Uint128::from(amount_to_borrow),
                };

                Ok(
                    Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: get_lending_contract(deps.as_ref()).unwrap(),
                        msg: to_binary(&msg_borrow_uncollateralazied)?,
                        funds: vec![],
                    })),
                )
            } else {
                Ok(
                    Response::default()
                )
            }
        }
        ExecuteMsg::CancelOrder { order_id } => {
            let order = get_order_by_id(deps.as_ref(), order_id).unwrap();

            let closed_order = OrderInfo {
                order_status: OrderStatus::Canceled,
                order_type: order.order_type,
                amount: order.amount,
                sell_token_denom: order.sell_token_denom,
                leverage: order.leverage,
            };

            MARGIN_POSITIONS.save(
                deps.storage,
                (info.sender.to_string(), order_id),
                &closed_order,
            )?;

            Ok(Response::default())
        }
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposit { user, denom } => to_binary(&get_deposit(deps, env, user, denom)?),
        QueryMsg::GetPrice { denom } => to_binary(&fetch_price_by_token(deps, env, denom)?),
        QueryMsg::GetOrdersByUser { user } => to_binary(&get_orders_by_user(deps, user)?),
        QueryMsg::GetOrderById { order_id } => to_binary(&get_order_by_id(deps, order_id)?),
        QueryMsg::GetLendingContract {} => to_binary(&get_lending_contract(deps)?),
        QueryMsg::GetCollateralVaultContract {} => to_binary(&get_collateral_vault_contract(deps)?),
    }
}

pub mod query {
    use crate::msg::OrderResponse;
    use crate::state::{MarginPositionsCount, COLLATERAL_VAULT, IS_TESTING, MARGIN_POSITIONS, MARGIN_POSITIONS_COUNT, PRICES, PRICE_FEED_IDS, PYTH_CONTRACT, SUPPORTED_TOKENS, USER_DEPOSITED_BALANCE, LENDING_CONTRACT};
    use cosmwasm_std::{Deps, Env, Order, StdResult, Uint128};
    use pyth_sdk_cw::{query_price_feed, PriceFeedResponse};

    pub fn get_deposit(deps: Deps, _env: Env, user: String, denom: String) -> StdResult<Uint128> {
        Ok(USER_DEPOSITED_BALANCE
            .load(deps.storage, (user, denom.clone()))
            .unwrap_or_else(|_| Uint128::zero()))
    }

    pub fn get_token_decimal(deps: Deps, denom: String) -> StdResult<Uint128> {
        // contract only inner call, so there is no need to parse non-existent token denom
        Ok(Uint128::from(
            SUPPORTED_TOKENS.load(deps.storage, denom).unwrap().decimals,
        ))
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

    pub fn get_collateral_vault_contract(deps: Deps) -> StdResult<String> {
        COLLATERAL_VAULT.load(deps.storage)
    }

    pub fn get_lending_contract(deps: Deps) -> StdResult<String> {
        LENDING_CONTRACT.load(deps.storage)
    }


    pub fn get_margin_positions_count(deps: Deps) -> StdResult<MarginPositionsCount> {
        MARGIN_POSITIONS_COUNT.load(deps.storage)
    }

    pub fn get_orders_by_user(deps: Deps, user: String) -> StdResult<Vec<OrderResponse>> {
        let all_positions: StdResult<Vec<_>> = MARGIN_POSITIONS
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        Ok(all_positions
            .unwrap()
            .iter()
            .filter(|((user_inner, _), _)| user == *user_inner)
            .map(|((_, _), order)| OrderResponse {
                order_status: order.order_status.clone(),
                order_type: order.order_type.clone(),
                amount: order.amount,
                sell_token_denom: order.sell_token_denom.clone(),
                leverage: order.leverage,
            })
            .collect())
    }

    pub fn get_order_by_id(deps: Deps, order_id: u128) -> StdResult<OrderResponse> {
        let all_positions: StdResult<Vec<_>> = MARGIN_POSITIONS
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        assert!(
            all_positions
                .unwrap()
                .iter()
                .any(|((_, order_id_inner), _)| *order_id_inner == order_id),
            "There is no such order with give order_id"
        );

        let mut result = vec![];

        let all_positions: StdResult<Vec<_>> = MARGIN_POSITIONS
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        for ((_, order_id_inner), order) in all_positions.unwrap().iter() {
            if *order_id_inner == order_id {
                result.push(OrderResponse {
                    order_status: order.order_status.clone(),
                    order_type: order.order_type.clone(),
                    amount: order.amount,
                    sell_token_denom: order.sell_token_denom.clone(),
                    leverage: order.leverage,
                })
            }
        }

        Ok(result.pop().unwrap())
    }
}
