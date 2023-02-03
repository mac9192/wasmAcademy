use cosmwasm_std::{Empty};
use cw_multi_test::{App, Contract, ContractWrapper};



use crate::{execute, instantiate, query};
use crate::multitest::CountingContact;


fn counting_contract() -> Box<dyn Contract<Empty>>{
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

const ATOM: &str = "atom";

#[test]
fn query_value() {
   
    
    use cosmwasm_std::Addr;
    use cosmwasm_std::Coin;

    let mut app = App::default();
    let sender = Addr::unchecked("sender");
    let contract_id = app.store_code(counting_contract());

    let contract = CountingContact::instantiate(
        &mut app,
        contract_id,
        &sender,
        None, 
        "Counting contract",
        Coin::new(10, ATOM),
    )
    .unwrap();

    let resp= contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn donate() {

    let mut app = App::default();

    let sender = Addr::unchecked("sender");
    let contract_id = app.store_code(counting_contract());

   
  
    use cosmwasm_std::Addr;

 
    use cosmwasm_std::Coin;


    
    let contract = CountingContact::instantiate(
        &mut app,
        contract_id,
        &sender,
        None,
        "Counting contract",
        Coin::new(10, ATOM),
    )
    .unwrap();
    
    contract.donate(&mut app, &sender, &[]).unwrap();
    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}


#[test]
fn donate_with_funds() {
    
    
    
 

     use cosmwasm_std::Addr;
    
   

     use cosmwasm_std::Coin;

     
     use cosmwasm_std::coins;
    
    
   // let mut app = App::default();
  
    let sender = Addr::unchecked("sender");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContact::instantiate(
        &mut app,
        contract_id,
        &sender,
        None,
        "Counting contract",
        Coin::new(10, ATOM),
    )
    .unwrap();
    
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 1 );
    
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(10, ATOM)
    );
   
}

#[test]
fn withdraw() {
    
  
  
  
    
     use cosmwasm_std::Addr;
  
   

     use cosmwasm_std::Coin;
    

 
     use cosmwasm_std::coins;
    
    
   // let mut app = App::default();
    
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &sender2, coins(5, ATOM))
            .unwrap();
    });

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContact::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        Coin::new(10, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender1, &coins(10, ATOM))
        .unwrap();

    contract
        .donate(&mut app, &sender2, &coins(5, ATOM))
        .unwrap();
    
    contract
        .withdraw(&mut app, &owner, &[]).unwrap();

   
    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(), 
        coins(15, ATOM)
    );

    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        vec![]
    );
    assert_eq!(
        app.wrap().query_all_balances(sender1).unwrap(),
        vec![]
    );
    assert_eq!(
        app.wrap().query_all_balances(sender2).unwrap(),
        vec![]
    );
   
}

#[test]
fn unauthorized_withdraw() {
    
 
     
     
     use cosmwasm_std::Addr;
  
  
    
     use cosmwasm_std::Coin;
    
     use crate::error::ContractError;


     

    
    
   // let mut app = App::default();
    
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");


    let mut app = App::default();

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContact::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        Coin::new(10, ATOM),
    )
    .unwrap();
    
        let err = contract.withdraw(&mut app, &member, &[]).unwrap_err();            
        assert_eq!(
            err,
            ContractError::Unauthorized {
                 owner: owner.into() 
                }, 
               
            );
   
    }