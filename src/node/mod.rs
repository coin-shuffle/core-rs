use std::marker::PhantomData;

use crate::node::storage::RoomStorage;

mod room;
mod storage;

#[derive(Debug, Clone)]
pub struct Node<R: RoomStorage> {
    shuffle_service_addr: String,
    room_storage: R,
    phantom: PhantomData<R>,
}

impl <R: RoomStorage> Node<R> {
    fn new(shuffle_service_addr: String, room_storage: R) -> Self {
        Self { 
            shuffle_service_addr, 
            room_storage,
            phantom: PhantomData,
        }
    }
}