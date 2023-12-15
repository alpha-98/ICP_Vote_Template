use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 5000;

/* 
    First thing to do in any smart contract is defining the types that
    we need to store in our state. OR
    This can be the return type of the functions as well as the arguments.
*/

/*
    For all of the `structs` we will drive the `candid type` and `deserialize`.
    We are doing this so that ICP will know how to deserialize it. AND
    We will use the default candid type for creating candid file and accessing below choice.
*/

// enums are only for return_types

#[derive(Debug, CandidType, Deserialize)]

enum Choice {
    Approve,
    Reject,
    Pass,
}

//     fn from_bytes(bytes: Cow<[u8]>) -> Self {
//         Decode!(bytes.as_ref(), Self).unwrap()
//     }
// }

// impl BoundedStorable {
//     const MAX_SIZE: u32 = MAX_VALUE_SIZE;
//     const IS_FIXED_SIZE: bool = false;
// }

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

}

// #[ic_cdk::query]
// fn get_proposal(key: u64) -> Option<Proposal> {}

// #[ic_cdk::query]
// fn get_proposal_count() -> u64 {}

// #[ic_cdk::update]
// fn create_proposal(key: u64, proposal: CreateProposal) -> Option<Proposal> {}

// #[ic_cdk::update]
// fn edit_proposal(key: u64, proposal: CreateProposal) -> Result<(), VoteError> {}

// #[ic_cdk::update]
// fn end_proposal(key: u64) -> Result<(), VoteError> {}

// #[ic_cdk::update]
// fn vote(key: u64, choice: Choice) -> Result<(), VoteError> {}
