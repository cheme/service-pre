//! Mpsc channel for usage with SRef

use immut_send::{
  SRef,
  SToRef,
};

use ::error::{
  Result,
};
use std::sync::mpsc::{
  Receiver as MpscReceiver,
//  TryRecvError,
  Sender as MpscSender,
  channel as mpsc_channel,
};

use ::{
  SpawnSend,
  SpawnRecv,
  SpawnChannel,
};


/// Mpsc channel as service send/recv, to use on Ref (non sendable content)
pub struct MpscChannelRef;
pub struct MpscSenderRef<C: SRef>(MpscSender<C::Send>);
pub struct MpscReceiverRef<C: SRef>(MpscReceiver<C::Send>);
pub struct MpscSenderToRef<CS>(MpscSender<CS>);
pub struct MpscReceiverToRef<CS>(MpscReceiver<CS>);

impl<C: SRef> SpawnSend<C> for MpscSenderRef<C> {
  const CAN_SEND: bool = <MpscSender<C::Send>>::CAN_SEND;
  fn send(&mut self, t: C) -> Result<()> {
    <MpscSender<C::Send> as SpawnSend<C::Send>>::send(&mut self.0, t.get_sendable())?;
    Ok(())
  }
}

impl<C: SRef> SpawnSend<C> for MpscSenderToRef<C::Send> {
  //impl<C : Ref<C>> ToRef<MpscSenderRef<C>,MpscSenderRef<C>> for MpscSender<C::Send> {
  const CAN_SEND: bool = true;
  fn send(&mut self, t: C) -> Result<()> {
    <MpscSender<C::Send>>::send(&mut self.0, t.get_sendable())?;
    Ok(())
  }
}

impl<C: SRef> SpawnRecv<C> for MpscReceiverRef<C> {
  fn recv(&mut self) -> Result<Option<C>> {
    let r = <MpscReceiver<C::Send> as SpawnRecv<C::Send>>::recv(&mut self.0)?;
    Ok(r.map(|tr| tr.to_ref()))
  }
}

impl<C: SRef> SpawnRecv<C> for MpscReceiverToRef<C::Send> {
  fn recv(&mut self) -> Result<Option<C>> {
    let r = <MpscReceiver<C::Send> as SpawnRecv<C::Send>>::recv(&mut self.0)?;
    Ok(r.map(|tr| tr.to_ref()))
  }
}

impl<C: SRef> Clone for MpscSenderRef<C> {
  fn clone(&self) -> Self {
    MpscSenderRef(self.0.clone())
  }
}

impl<C: SRef> SRef for MpscSenderRef<C> {
  type Send = MpscSenderToRef<C::Send>;
  #[inline]
  fn get_sendable(self) -> Self::Send {
    MpscSenderToRef(self.0)
  }
}

impl<C: SRef> SToRef<MpscSenderRef<C>> for MpscSenderToRef<C::Send> {
  #[inline]
  fn to_ref(self) -> MpscSenderRef<C> {
    MpscSenderRef(self.0)
  }
}

impl<C: SRef> SRef for MpscReceiverRef<C> {
  type Send = MpscReceiverToRef<C::Send>;
  #[inline]
  fn get_sendable(self) -> Self::Send {
    MpscReceiverToRef(self.0)
  }
}

impl<C: SRef> SToRef<MpscReceiverRef<C>> for MpscReceiverToRef<C::Send> {
  #[inline]
  fn to_ref(self) -> MpscReceiverRef<C> {
    MpscReceiverRef(self.0)
  }
}

impl<C: SRef> SpawnChannel<C> for MpscChannelRef {
  type WeakSend = MpscSenderRef<C>;
  type Send = MpscSenderRef<C>;
  type Recv = MpscReceiverRef<C>;
  fn new(&mut self) -> Result<(Self::Send, Self::Recv)> {
    let (s, r) = mpsc_channel();
    Ok((MpscSenderRef(s), MpscReceiverRef(r)))
  }
  fn get_weak_send(s: &Self::Send) -> Option<Self::WeakSend> {
    Some(s.clone())
  }
}

