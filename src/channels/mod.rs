//! Cross implementation channels


use error::{
  Result,
};


use super::{
  SpawnSend,
  SpawnUnyield,
};

use std::collections::VecDeque;

mod default;

pub use self::default::{
  DefaultRecv,
  DefaultRecvRef,
  DefaultRecvChannel,
};
mod evented;
pub use self::evented::{
  MioRecv,
  MioSend,
};
pub mod void;
pub mod mpsc;
pub mod mpscref;
pub mod rc;

#[cfg(feature = "mio-transport")]
pub mod mio;




/// Local spawn send, could be use for non thread spawn (not Send)
/// Not for local coroutine as Rc is required in this case
/// TODO find use case
impl<'a, C> SpawnSend<C> for &'a mut VecDeque<C> {
  const CAN_SEND: bool = true;
  fn send(&mut self, c: C) -> Result<()> {
    self.push_front(c);
    Ok(())
  }
}



/// struct to combine send channel and an unyield channel:
/// for the common use case where we unyield on every send.
#[derive(Clone)]
pub struct HandleSend<S, H: SpawnUnyield>(pub S, pub H);


impl<C, S: SpawnSend<C>, H: SpawnUnyield> SpawnSend<C> for HandleSend<S, H> {
  const CAN_SEND: bool = <S as SpawnSend<C>>::CAN_SEND;
  fn send(&mut self, t: C) -> Result<()> {
    self.0.send(t)?;
    self.1.unyield()?;
    Ok(())
  }
}



/// return the command if handle is finished
#[inline]
pub fn send_with_handle<S: SpawnSend<C>, H: SpawnUnyield, C>(
  s: &mut S,
  h: &mut H,
  command: C,
) -> Result<Option<C>> {
  //Service,Sen,Recv
  Ok(if h.is_finished() || !<S as SpawnSend<C>>::CAN_SEND {
    Some(command)
  } else {
    s.send(command)?;
    h.unyield()?;
    None
  })
}

/// tech trait only for implementing send with handle as member of a struct (avoid unconstrained
/// parameter for HandleSend but currently use only for this struct)
pub trait SpawnSendWithHandle<C> {
  fn send_with_handle(&mut self, C) -> Result<Option<C>>;
}


impl<C, S: SpawnSend<C>, H: SpawnUnyield> SpawnSendWithHandle<C> for HandleSend<S, H> {
  #[inline]
  fn send_with_handle(&mut self, command: C) -> Result<Option<C>> {
    send_with_handle(&mut self.0, &mut self.1, command)
  }
}


#[cfg(test)]
fn test_trait_item1<M>(_i: Box<SpawnSendWithHandle<M>>) {}

#[cfg(test)]
fn test_trait_item2<M>(_i: &SpawnSendWithHandle<M>) {}

