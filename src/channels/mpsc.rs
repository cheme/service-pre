use std::sync::mpsc::{
  Receiver as MpscReceiver,
  TryRecvError,
  Sender as MpscSender,
  channel as mpsc_channel,
};


use ::error::{
  Result,
};

use ::{
  SpawnRecv,
  SpawnSend,
  SpawnChannel,
};

impl<C> SpawnRecv<C> for MpscReceiver<C> {
  fn recv(&mut self) -> Result<Option<C>> {
    match self.try_recv() {
      Ok(t) => Ok(Some(t)),
      Err(e) => if let TryRecvError::Empty = e {
        Ok(None)
      } else {
        Err(e.into()) // ErrorKind::ChannelTryRecvError,
      },
    }
  }
}

impl<C> SpawnSend<C> for MpscSender<C> {
  const CAN_SEND: bool = true;
  fn send(&mut self, t: C) -> Result<()> {
    <MpscSender<C>>::send(self, t)?;
    Ok(())
  }
}

/// Mpsc channel as service send/recv
pub struct MpscChannel;

impl<C: Send> SpawnChannel<C> for MpscChannel {
  type WeakSend = MpscSender<C>;
  type Send = MpscSender<C>;
  type Recv = MpscReceiver<C>;
  /// new use a struct as input to allow construction over a MyDht for a Peer :
  /// TODO MDht peer chan : first ensure peer is connected (or not) then run new channel -> send in
  /// sync send of loop -> loop proxy to peer asynchronously -> dist peer receive command if
  /// connected, then send in Read included service dest (their spawn impl is same as ours) -> des
  /// service send to our peer send which send to us -> the recv got a dest to this recv
  /// => this for this case we create two under lying channel from out of loop to loop and from
  /// readpeer to out of loop. Second channel is needed to start readservice : after connect ->
  /// connect is done with this channel creation (cf slab cache)
  fn new(&mut self) -> Result<(Self::Send, Self::Recv)> {
    let chan = mpsc_channel();
    Ok(chan)
  }
  fn get_weak_send(s: &Self::Send) -> Option<Self::WeakSend> {
    Some(s.clone())
  }
}

