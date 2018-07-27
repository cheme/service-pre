use ::error::{
  Result,
};
use ::{
  SpawnRecv,
  SpawnSend,
};
use ::eventloop::{
  TriggerReady,
  Registerable,
  Token,
  Ready,
};

/// event loop registrable channel receiver
/// TODO rename without mio ref
pub struct MioRecv<R, H> {
  pub mpsc: R,
  pub reg: H,
}


/// mio registerable channel sender
#[derive(Clone)]
pub struct MioSend<S, T> {
  pub mpsc: S,
  pub set_ready: T,
}

impl<C, R: SpawnRecv<C>, H> SpawnRecv<C> for MioRecv<R, H> {
  fn recv(&mut self) -> Result<Option<C>> {
    self.mpsc.recv()
  }
}


impl<C, S: SpawnSend<C>, T: TriggerReady> SpawnSend<C> for MioSend<S, T> {
  const CAN_SEND: bool = <S as SpawnSend<C>>::CAN_SEND;
  fn send(&mut self, t: C) -> Result<()> {
    self.mpsc.send(t)?;
    self.set_ready.set_readiness(Ready::Readable)?;
    Ok(())
  }
}


impl<PO, R, H: Registerable<PO>> Registerable<PO> for MioRecv<R, H> {
  fn register(&self, p: &PO, t: Token, r: Ready) -> Result<bool> {
    self.reg.register(p, t, r)
  }
  fn reregister(&self, p: &PO, t: Token, r: Ready) -> Result<bool> {
    self.reg.reregister(p, t, r)
  }
  fn deregister(&self, p: &PO) -> Result<()> {
    self.reg.deregister(p)
  }
}

