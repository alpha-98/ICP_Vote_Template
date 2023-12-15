use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::BTreeMap;
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

/*
    It's OPTIONAL!
    We have VoteError so front-end know what went wrong
    in case we have face any problem.
*/
#[derive(Debug, CandidType, Deserialize)]

enum VoteError {
    AlreadyVoted,
    ProposalIsNotActive,
    NoSuchProposal,
    AccessRejected,
    UpdateError,
}

/*
    Create actual Propsal itself.
    Principal is what stands as a wallet address in ICP.
*/
#[derive(Debug, CandidType, Deserialize)]

struct Proposal {
    description: String,
    approve: u32,
    reject: u32,
    pass: u32,
    is_active: bool,
    voted: Vec<candid::Principal>, // Vector of the user who have voted for this proposal.
    owner: candid::Principal, // Owner of propsal and candid principal and SYNTAX of accessing principal.
}

#[derive(Debug, CandidType, Deserialize)]
/* 
    create propsal is justfor an argument type. SO
    We don't need to store it in Storable.
*/
struct CreateProposal {
    description: String,
    is_active: bool,
}

/*
    We are implementing the storable for the state we are going to store.
    Inside our state we are going to hold Proposal struct.
*/
impl Storable for Proposal {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Proposal {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false; // for stable binary tree where we are going to store the proposal object.
}

/*
    Thread local esures that we are dealing with our local thread.
    Since ICP smart contract are not multi-threaded we will just be working on our local thread.
*/
thread_local! {
    // Memory manager is going to be the same for most of the smart contracts that we're going to implement.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // For Storing the Proposal map.
    // It's enusre that our state is going to be preserved among updates.
    static PROPOSAL_MAP: RefCell<StableBTreeMap<u64,Proposal,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))));

}

#[ic_cdk::query]
fn get_proposal(key: u64) -> Option<Proposal> {
    PROPOSAL_MAP.with(|p| p.borrow().get(&key))
}

#[ic_cdk::query]
fn get_proposal_count() -> u64 {
    PROPOSAL_MAP.with(|p| p.borrow().len())
}

#[ic_cdk::update]
fn Create_proposal(key: u64, proposal: CreateProposal) -> Option<Proposal> {
    let value: Proposal = Proposal {
        description: proposal.description,
        approve: 0u32,
        reject: 0u32,
        pass: 0u32,
        is_active: proposal.is_active,
        voted: vec![],
        owner: ic_cdk::caller(),
    };

    PROPOSAL_MAP.with(|p| p.borrow_mut().insert(key, value))
}

#[ic_cdk::update]
fn edit_proposal(key: u64, proposal: CreateProposal) -> Result<(), VoteError> {
    PROPOSAL_MAP.with(|p| {
        let old_proposal_opt = p.borrow().get(&key);
        let old_proposal: Proposal;

        match old_proposal_opt {
            Some(value) => old_proposal = value,
            None => return Err(VoteError::NoSuchProposal),
        }

        if old_proposal.owner != ic_cdk::caller() {
            return Err(VoteError::AccessRejected);
        }

        let value: Proposal = Proposal {
            description: proposal.description,
            approve: old_proposal.approve,
            reject: old_proposal.reject,
            pass: old_proposal.pass,
            is_active: proposal.is_active,
            voted: old_proposal.voted,
            owner: old_proposal.owner,
        };

        let res: Option<Proposal> = p.borrow_mut().insert(key, value);

        match res {
            Some(_) => Ok(()),
            None => Err(VoteError::UpdateError),
        }
    })
}

#[ic_cdk::update]
fn end_proposal(key: u64) -> Result<(), VoteError> {
    PROPOSAL_MAP.with(|p| {
        let old_proposal_opt = p.borrow().get(&key);
        let mut old_proposal: Proposal;

        match old_proposal_opt {
            Some(value) => old_proposal = value,
            None => return Err(VoteError::NoSuchProposal),
        }

        if old_proposal.owner != ic_cdk::caller() {
            return Err(VoteError::AccessRejected);
        }

        old_proposal.is_active = false;

        let res: Option<Proposal> = p.borrow_mut().insert(key, old_proposal);

        match res {
            Some(_) => Ok(()),
            None => Err(VoteError::UpdateError),
        }
    })
}

#[ic_cdk::update]
fn vote(key: u64, choice: Choice) -> Result<(), VoteError> {
    PROPOSAL_MAP.with(|p| {
        let proposal_opt: Option<Proposal> = p.borrow().get(&key);
        let mut proposal: Proposal;

        match proposal_opt {
            Some(value) => proposal = value,
            None => return Err(VoteError::NoSuchProposal),
        }

        let caller: Principal = ic_cdk::caller();

        if proposal.voted.contains(&caller) {
            return Err(VoteError::AlreadyVoted);
        } else if proposal.is_active != true {
            return Err(VoteError::ProposalIsNotActive);
        }

        match choice {
            Choice::Approve => proposal.approve += 1,
            Choice::Pass => proposal.pass -= 1,
            Choice::Reject => proposal.reject += 1,
        }

        proposal.voted.push(caller);
        let res: Option<Proposal> = p.borrow_mut().insert(key, proposal);

        match res {
            Some(_) => Ok(()),
            None => Err(VoteError::UpdateError),
        }
    })
}