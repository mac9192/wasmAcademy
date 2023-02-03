/*query handler */
use cosmwasm_std::{ DepsMut, StdResult, Response, MessageInfo, Coin };
use cw_storage_plus::Item;

use crate::msg::InstantiateMsg;
use crate::state::{OWNER, STATE, State};
use crate::error::ContractError;
use cw2::{set_contract_version, get_contract_version};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {

   set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?; 

   STATE.save(
       deps.storage,
       &State {
           counter: 0,
           minimal_donation: msg.minimal_donation,
       },
   )?;
    OWNER.save(deps.storage, &info.sender)?; 

    Ok(Response::new())
}

pub fn migrate(mut deps: DepsMut) -> Result<Response, ContractError> {
    let contract = get_contract_version(deps.storage)?;
    if contract.contract != CONTRACT_NAME {
        return Err(ContractError::InvalidName(contract.contract))
    }

    let resp = match contract.version.as_str(){
        "0.1.0" => migrate_0_1_0(deps.branch())?,
        CONTRACT_VERSION => return Ok(Response::new()),
        _ => return Err(ContractError::InvalidVersion(contract.version.to_string())),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

   Ok(resp)
}

pub fn migrate_0_1_0(deps: DepsMut) ->StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");

    let counter = COUNTER.load(deps.storage)?;
    let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;

    STATE.save(
        deps.storage,
        &State {
            counter, 
            minimal_donation
        },
    )?;

    Ok(Response::new())
}



pub mod query {
    use crate::msg::ValueResp;
    use crate::state::STATE;
    use cosmwasm_std::{Deps, StdResult}; 
    
    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = STATE.load(deps.storage)?.counter;
       
        Ok(ValueResp { value })
    }
}

pub mod exec {
    use cosmwasm_std::{DepsMut, Response, StdResult, MessageInfo, Env,  BankMsg};
    use crate::state::{STATE, OWNER};
    use crate::error::ContractError;
    

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        
        let mut state = STATE.load(deps.storage)?;
    

        //Maybe hoe to update Meta : COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

  

        if info.funds.iter().any(|coin| {
            coin.denom == state.minimal_donation.denom && coin.amount >= state.minimal_donation.amount
        }){
            state.counter += 1;
            STATE.save(deps.storage, &state)?;
        }

    
        let resp = Response::new()
        .add_attribute("action", "poke")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("counter", state.counter.to_string());
        Ok(resp)
    }


    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        
        use crate::error::ContractError;
 
        
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized { 
                owner: owner.into(),
             });
        }
    
        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: balance,
        };
    
        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());
    
        Ok(resp)
    }
}