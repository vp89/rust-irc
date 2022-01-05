use std::collections::VecDeque;
use std::ops::AddAssign;

use async_trait::async_trait;

#[async_trait]
pub trait ReceiverWrapper<T> {
    async fn receive(&mut self) -> Option<T>;
}

pub struct FakeChannelReceiver<T>
where
    T: Send + Sync,
{
    pub faked_messages: Box<VecDeque<T>>,
    pub receive_count: i32,
}

#[async_trait]
impl<T> ReceiverWrapper<T> for tokio::sync::mpsc::Receiver<T>
where
    T: Send,
{
    async fn receive(&mut self) -> Option<T> {
        self.recv().await
    }
}

#[async_trait]
impl<T> ReceiverWrapper<T> for FakeChannelReceiver<T>
where
    T: Send + Sync,
{
    async fn receive(&mut self) -> Option<T> {
        self.receive_count.add_assign(1);
        self.faked_messages.pop_front()
    }
}
