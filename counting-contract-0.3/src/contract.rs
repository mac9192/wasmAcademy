/*query handler */
#![allow(unused)]
#![allow(dead_code)]
use cosmwasm_std::{ DepsMut, StdResult, Response, MessageInfo, Coin, Decimal, WasmMsg, Env, to_binary };
use cw_storage_plus::Item;

use crate::msg::{InstantiateMsg, MigrateMsg, Parent, ExecMsg};
use crate::state::{OWNER, STATE, State, ParentDonation, PARENT_DONATION};
use crate::error::ContractError;
use serde::{Deserialize, Serialize};

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
           donating_parent: msg.parent.as_ref().map(|p| p.donating_period),
       },
   )?;
    OWNER.save(deps.storage, &info.sender)?; 

   if let Some(parent) = msg.parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            }
        )?;
   }

    Ok(Response::new())
}

pub fn migrate(mut deps: DepsMut, msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract = get_contract_version(deps.storage)?;
    if contract.contract != CONTRACT_NAME {
        return Err(ContractError::InvalidName(contract.contract))
    }

    let resp = match contract.version.as_str(){
        "0.1.0" => migrate_0_1_0(deps.branch(), msg.parent)?,
        "0.2.0" => migrate_0_2_0(deps.branch(), msg.parent)?,
        version if version == CONTRACT_VERSION => return Ok(Response::new()),
        _ => return Err(ContractError::InvalidVersion(contract.version.to_string())),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

   Ok(resp)
}

pub fn migrate_0_1_0(deps: DepsMut, parent: Option<Parent>) ->StdResult<Response> {



    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");


    let counter = COUNTER.load(deps.storage)?;
    let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;


  STATE.save(deps.storage, &State {
      counter, 
      minimal_donation,
      donating_parent: parent.as_ref().map(|p| p.donating_period),
  })?;

    if let Some(parent) = parent{
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            }
        )?;
    }
           Ok(Response::new())



}


pub fn migrate_0_2_0(deps: DepsMut, parent: Option<Parent>) ->StdResult<Response> {

    #[derive(Deserialize, Serialize)]
    struct OldState {
        counter: u64,
        minimal_donation: Coin,
    }

   
    const OLD_STATE: Item<OldState> = Item::new("state");

 
    let state = OLD_STATE.load(deps.storage)?;

     STATE.save (
      deps.storage,
      &State {
          counter: state.counter,
          minimal_donation: state.minimal_donation,
          donating_parent: parent.as_ref().map(|p| p.donating_period),
      },
     )?;

  

    if let Some(parent) = parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            }
        )?;
    }

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
    

    pub fn donate(deps: DepsMut, info: MessageInfo, env: Env) -> StdResult<Response> {

        use crate::state::PARENT_DONATION;
        use cosmwasm_std::{to_binary, WasmMsg};
        use crate::msg::ExecMsg;

        
        let mut state = STATE.load(deps.storage)?;
        let mut resp = Response::default();
    

        //Maybe hoe to update Meta : COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

  

        if info.funds.iter().any(|coin| {
            coin.denom == state.minimal_donation.denom && coin.amount >= state.minimal_donation.amount
        }){
            state.counter += 1;

            if let Some(parent) = &mut state.donating_parent {
                *parent -= 1;

                if *parent == 0 {
                    let parent_donation = PARENT_DONATION.load(deps.storage)?;
                    *parent = parent_donation.donating_parent_period; 

                    let funds = deps.querier.query_all_balances(env.contract.address)?
                    .into_iter()
                    .map(|mut coin| {
                        coin.amount = coin.amount * parent_donation.part;
                        coin
                    })
                    .collect();

                    let msg = WasmMsg::Execute {
                        contract_addr: parent_donation.address.to_string(),
                        msg: to_binary(&ExecMsg::Donate {})?,
                        funds,
                    };

                 
                    resp = resp
                        .add_message(msg)
                        .add_attribute("donated_to_parent", parent_donation.address.to_string());
                } 
            }

            STATE.save(deps.storage, &state)?;  
        }

    
       resp = resp
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