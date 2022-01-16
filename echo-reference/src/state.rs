use borsh::{BorshDeserialize, BorshSerialize};
use std::mem::size_of;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AuthorizedBufferHeader {
    pub bump_seed: u8,
    pub buffer_seed: u64,
}

pub const AUTH_BUFF_HEADER_SIZE: usize = size_of::<u8>() + size_of::<u64>();

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VendingMachineBufferHeader {
    pub bump_seed: u8,
    pub price: u64,
}

pub const VENDING_MACHINE_BUFF_HEADER_SIZE: usize = size_of::<u8>() + size_of::<u64>();
