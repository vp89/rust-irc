use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::AddAssign;

use async_trait::async_trait;

#[async_trait]
pub trait ReceiverWrapper<T>
where
    T: Sync + Send,
{
    async fn receive(&mut self) -> Option<T>;
}

pub struct FakeMessagesWrapper<T>(pub RefCell<Box<VecDeque<T>>>);
pub struct FakeReceiveCountWrapper(pub RefCell<i32>);

// https://stackoverflow.com/questions/36649865/how-can-i-guarantee-that-a-type-that-doesnt-implement-sync-can-actually-be-safe
// This wrapper is only used for unit testing and I don't know how
// to use RefCell safely, maybe revisit if RefCell is still required in order
// to remove this? But not a big deal if unit test code uses unsafe.
unsafe impl<T> Sync for FakeMessagesWrapper<T> {}
unsafe impl Sync for FakeReceiveCountWrapper {}

pub struct FakeChannelReceiver<T>
where
    T: Send + Sync,
{
    pub faked_messages: FakeMessagesWrapper<T>,
    pub receive_count: FakeReceiveCountWrapper,
}

#[async_trait]
impl<T> ReceiverWrapper<T> for tokio::sync::mpsc::Receiver<T>
where
    T: Send + Sync,
{
    async fn receive(&mut self) -> Option<T> {
        // ClientToServerChannelFailedToReceive
        self.recv().await
    }
}

#[async_trait]
impl<T> ReceiverWrapper<T> for FakeChannelReceiver<T>
where
    T: Clone + Send + Sync,
{
    async fn receive(&mut self) -> Option<T> {
        self.receive_count.0.borrow_mut().add_assign(1);
        self.faked_messages.0.borrow_mut().pop_front()
    }
}
