use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::{AddAssign};

use crate::result::Result;
use crate::error::Error::{ClientToServerChannelFailedToReceive,TestErrorNoMoreMessagesInReceiver};

pub trait ReceiverWrapper<T> {
    fn receive(&self) -> Result<T>;
}

pub struct FakeChannelReceiver<T> {
    pub faked_messages: RefCell<Box<VecDeque<T>>>,
    pub receive_count: RefCell<i32>
}

impl<T> ReceiverWrapper<T> for std::sync::mpsc::Receiver<T> {
    fn receive(&self) -> Result<T> {
        Ok(self.recv().map_err(ClientToServerChannelFailedToReceive)?)
    }
}

impl<T> ReceiverWrapper<T> for FakeChannelReceiver<T> where
    T: Clone {
    fn receive(&self) -> Result<T> {
        self.receive_count.borrow_mut().add_assign(1);
        let foo = self.faked_messages.borrow_mut().pop_front().ok_or_else(|| TestErrorNoMoreMessagesInReceiver)?.clone();
        Ok(foo)
    }
}
