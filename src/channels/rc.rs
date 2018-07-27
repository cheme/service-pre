use ::{
  SpawnSend,
  SpawnRecv,
  SpawnChannel,
};

use super::void::NoSend;

use ::error::{
  Result,
};

use std::rc::Rc;

use std::cell::RefCell;

use std::collections::VecDeque;
  

pub struct LocalRcChannel;

/// common send/recv for coroutine local usage (not Send, but clone)
pub type LocalRc<C> = Rc<RefCell<VecDeque<C>>>;


impl<C> SpawnChannel<C> for LocalRcChannel {
  type WeakSend = NoSend;
  type Send = LocalRc<C>;
  type Recv = LocalRc<C>;
  fn new(&mut self) -> Result<(Self::Send, Self::Recv)> {
    let lr = Rc::new(RefCell::new(VecDeque::new()));
    Ok((lr.clone(), lr))
  }
  fn get_weak_send(_: &Self::Send) -> Option<Self::WeakSend> {
    None
  }
}
impl<C> SpawnSend<C> for LocalRc<C> {
  const CAN_SEND: bool = true;
  fn send(&mut self, c: C) -> Result<()> {
    self.borrow_mut().push_front(c);
    Ok(())
  }
}
impl<C> SpawnRecv<C> for LocalRc<C> {
  fn recv(&mut self) -> Result<Option<C>> {
    Ok(self.borrow_mut().pop_back())
  }
}

