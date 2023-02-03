#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ to_binary, Binary, DepsMut, Deps, Env, MessageInfo, Response, StdResult, Empty};
use msg::InstantiateMsg;
use cosmwasm_std::Coin;
use crate::error::ContractError;


pub mod msg;
mod contract;
mod state;
pub mod error;
#[cfg(any(test, feature = "tests"))]
pub mod multitest; 

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate (
    deps: DepsMut, 
    _env: Env, 
    info: MessageInfo, 
    msg: InstantiateMsg,
)  -> Result<Response, ContractError> {
    contract::instantiate(deps, info, msg);
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query( deps: Deps , _env: Env, msg: msg::QueryMsg )  -> StdResult<Binary> {
   use msg::QueryMsg::*;

   match msg {
       Value {} => to_binary(&contract::query::value(deps)?),
   }
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute (
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    msg: msg::ExecMsg,
)  -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match msg {
        Donate {} => contract::exec::donate(deps, info).map_err(ContractError::from),
        Withdraw {} => contract::exec::withdraw(deps, env, info),
    }  
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    contract::migrate(deps)
}
