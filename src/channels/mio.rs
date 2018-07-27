//! Implement channel with registration in mio event loop

extern crate mio;


use ::error::{
  Result,
};
use super::{
  MioSend,
  MioRecv,
};

use ::{
  SpawnChannel,
};

use self::mio::{
  Registration,
  SetReadiness,
};

/// channel register on mio poll in service (service constrains Registrable on
/// This allows running mio loop as service, note that the mio loop is reading this channel, not
/// the spawner loop (it can for commands like stop or restart yet it is not the current usecase
/// (no need for service yield right now).
pub struct MioChannel<CH>(pub CH);

impl<C, CH: SpawnChannel<C>> SpawnChannel<C> for MioChannel<CH> {
  type WeakSend = MioSend<CH::WeakSend, SetReadiness>;
  type Send = MioSend<CH::Send, SetReadiness>;
  type Recv = MioRecv<CH::Recv, Registration>;
  fn new(&mut self) -> Result<(Self::Send, Self::Recv)> {
    let (s, r) = self.0.new()?;
    let (reg, sr) = Registration::new2();
    Ok((
      MioSend {
        mpsc: s,
        set_ready: sr,
      },
      MioRecv { mpsc: r, reg: reg },
    ))
  }
  fn get_weak_send(s: &Self::Send) -> Option<Self::WeakSend> {
    <CH as SpawnChannel<C>>::get_weak_send(&s.mpsc).map(|ms| MioSend {
      mpsc: ms,
      set_ready: s.set_ready.clone(),
    })
  }
}

