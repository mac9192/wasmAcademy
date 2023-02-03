use cw_storage_plus::Item;
use cosmwasm_std::{Coin, Addr, Decimal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub counter: u64,
    pub minimal_donation: Coin,
    pub donating_parent: Option<u64>, 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ParentDonation {
    pub address: Addr, 
    pub donating_parent_period: u64,
    pub part: Decimal,
}

pub const PARENT_DONATION: Item<ParentDonation> = Item::new("parent_donation");
pub const STATE: Item<State> = Item::new("state");
pub const OWNER: Item<Addr> = Item::new("owner");