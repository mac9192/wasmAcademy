/*query handler */
#![allow(unused)]
#![allow(dead_code)]
use cosmwasm_std::{ DepsMut, StdResult, Response, MessageInfo };

use crate::state::COUNTER;
use crate::msg::InstantiateMsg;
use crate::state::{OWNER, MINIMAL_DONATION};
use crate::error::ContractError;
use cw2::set_contract_version;


const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    COUNTER.save(deps.storage, &0)?;
    MINIMAL_DONATION.save(deps.storage, &msg.minimal_donation)?;
    OWNER.save(deps.storage, &info.sender)?; 

    Ok(Response::new())
}

pub mod query {
    use crate::msg::ValueResp;
    use crate::state::COUNTER;
    use cosmwasm_std::{Deps, StdResult};
    
    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = COUNTER.load(deps.storage)?;
       
        Ok(ValueResp { value })
    }
}

pub mod exec {
    use cosmwasm_std::{DepsMut, Response, StdResult, MessageInfo, Env,  BankMsg};
    use crate::state::{COUNTER, MINIMAL_DONATION, OWNER}  ;
    

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let minimal_donation = MINIMAL_DONATION.load(deps.storage)?; 

        //Maybe hoe to update Meta : COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

        let mut value = COUNTER.load(deps.storage)?;

        if info.funds.iter().any(|coin| {
            coin.denom == minimal_donation.denom && coin.amount >= minimal_donation.amount
        }){
            value += 1;
            COUNTER.save(deps.storage, &value)?;
        }

        COUNTER.save(deps.storage, &value)?;

        let resp = Response::new()
        .add_attribute("action", "poke")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("counter", value.to_string());
        Ok(resp)
    }

    use crate::ContractError;
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