use immut_send::{
  SRef,
  SToRef,
};

use ::{
  SpawnSend,
  SpawnRecv,
  SpawnChannel,
};

use ::error::{
  Result,
  ErrorKind,
};

#[derive(Clone, Debug)]
pub struct NoChannel;
#[derive(Clone, Debug)]
pub struct NoRecv;
#[derive(Clone, Debug)]
pub struct NoSend;


sref_self!(NoSend);

sref_self!(NoRecv);


impl<C> SpawnChannel<C> for NoChannel {
  type WeakSend = NoSend;
  type Send = NoSend;
  type Recv = NoRecv;
  fn new(&mut self) -> Result<(Self::Send, Self::Recv)> {
    Ok((NoSend, NoRecv))
  }
  fn get_weak_send(_: &Self::Send) -> Option<Self::WeakSend> {
    None
  }
}
impl<C> SpawnRecv<C> for NoRecv {
  #[inline]
  fn recv(&mut self) -> Result<Option<C>> {
    Ok(None)
  }

  //TODO fn close(&mut self) -> Result<()> : close receiver, meaning senders will fail on send and
  //could be drop TODO also require is_close function (same for send) TODO this mus be last
}

impl<C> SpawnSend<C> for NoSend {
  const CAN_SEND: bool = false;
  #[inline]
  fn send(&mut self, _: C) -> Result<()> {
    // return as Bug : code should check CAN_SEND before
    Err(ErrorKind::Bug("Spawner does not support send".to_string()).into()) // ErrorKind::Bug
  }
}

