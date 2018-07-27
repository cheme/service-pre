//! Set a default value on receiver, changing the service using the receiver in a infinite loop.
//! Some other inner service function will yield if needed (eg a tcp server
//! reading incoming connections)


use error::{
  Result,
};


use ::{
  SpawnRecv,
  SpawnChannel,
};

use immut_send::{
  SRef,
  SToRef,
};
use rust_proto::Proto;

/// set a default value to receiver (spawn loop will therefore not yield on receiver
pub struct DefaultRecvChannel<C, CH>(pub CH, pub C);

/// set a default value to receiver (spawn loop will therefore not yield on receiver
pub struct DefaultRecv<C, SR>(pub SR, pub C);
pub struct DefaultRecvRef<C: SRef, SR: SRef>(DefaultRecv<<C as SRef>::Send, <SR as SRef>::Send>);


impl<C: Proto, CH: SpawnChannel<C>> SpawnChannel<C> for DefaultRecvChannel<C, CH> {
  type WeakSend = CH::WeakSend;
  type Send = CH::Send;
  type Recv = DefaultRecv<C, CH::Recv>;

  fn new(&mut self) -> Result<(Self::Send, Self::Recv)> {
    let (s, r) = self.0.new()?;
    Ok((s, DefaultRecv(r, self.1.get_new())))
  }
  fn get_weak_send(s: &Self::Send) -> Option<Self::WeakSend> {
    <CH as SpawnChannel<C>>::get_weak_send(s)
  }
}

impl<C: Proto, SR: SpawnRecv<C>> SpawnRecv<C> for DefaultRecv<C, SR> {
  #[inline]
  fn recv(&mut self) -> Result<Option<C>> {
    let r = self.0.recv();
    if let Ok(None) = r {
      return Ok(Some(self.1.get_new()));
    }
    r
  }
}

impl<C: SRef, SR: SRef> SRef for DefaultRecv<C, SR> {
  type Send = DefaultRecvRef<C, SR>;
  #[inline]
  fn get_sendable(self) -> Self::Send {
    DefaultRecvRef(DefaultRecv(self.0.get_sendable(), self.1.get_sendable()))
  }
}

impl<C: SRef, SR: SRef> SToRef<DefaultRecv<C, SR>> for DefaultRecvRef<C, SR> {
  #[inline]
  fn to_ref(self) -> DefaultRecv<C, SR> {
    DefaultRecv((self.0).0.to_ref(), (self.0).1.to_ref())
  }
}

impl<C: SRef, SR: SRef> SpawnRecv<C> for DefaultRecvRef<C, SR>
where
  <C as SRef>::Send: Proto,
  <SR as SRef>::Send: SpawnRecv<<C as SRef>::Send>,
{
  #[inline]
  fn recv(&mut self) -> Result<Option<C>> {
    let a = self.0.recv()?;
    Ok(a.map(|a| a.to_ref()))
  }
}

