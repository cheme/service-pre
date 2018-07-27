extern crate mio;

use self::mio::net::TcpStream as MioStream;
//use std::net::{SocketAddr, Shutdown};
use ::error::{
  Result,
};

use self::mio::{
  Events as MioEventsInner,
  SetReadiness,
  Evented,
  Poll as MioPoll,
  Token as MioToken,
  Ready as MioReady,
//  Event as MioEvent,
  PollOpt,
};
use super::*;

pub struct MioEvents(pub MioEventsInner, pub usize);

impl Events for MioEvents {
  fn with_capacity(s: usize) -> Self {
    MioEvents(MioEventsInner::with_capacity(s), 0)
  }
}

impl Iterator for MioEvents {
  type Item = Event;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(event) = self.0.get(self.1) {
      self.1 += 1;
      if event.readiness().is_readable() {
        Some(Event {
          kind: Ready::Readable,
          token: event.token().0,
        })
      } else if event.readiness().is_writable() {
        Some(Event {
          kind: Ready::Writable,
          token: event.token().0,
        })
      } else {
        // skip
        self.next()
      }
    } else {
      None
    }
  }
}

#[derive(Clone)]
pub struct MioEvented<H>(pub H);

impl<H: Evented> Registerable<MioPoll> for MioEvented<H> {
  fn register(&self, p: &MioPoll, t: Token, r: Ready) -> Result<bool> {
    match r {
      Ready::Readable => p.register(&self.0, MioToken(t), MioReady::readable(), PollOpt::edge())?,
      Ready::Writable => p.register(&self.0, MioToken(t), MioReady::writable(), PollOpt::edge())?,
    }
    Ok(true)
  }
  fn reregister(&self, p: &MioPoll, t: Token, r: Ready) -> Result<bool> {
    match r {
      Ready::Readable => p.reregister(&self.0, MioToken(t), MioReady::readable(), PollOpt::edge())?,
      Ready::Writable => p.reregister(&self.0, MioToken(t), MioReady::writable(), PollOpt::edge())?,
    }
    Ok(true)
  }
  fn deregister(&self, poll: &MioPoll) -> Result<()> {
    poll.deregister(&self.0)?;
    Ok(())
  }
}


impl TriggerReady for SetReadiness {
  fn set_readiness(&self, ready: Ready) -> Result<()> {
    match ready {
      Ready::Readable => SetReadiness::set_readiness(&self, MioReady::readable())?,
      Ready::Writable => SetReadiness::set_readiness(&self, MioReady::writable())?,
    }
    Ok(())
  }
}

impl Poll for MioPoll {
  type Events = MioEvents;
  fn poll(&self, events: &mut Self::Events, timeout: Option<Duration>) -> Result<usize> {
    let r = MioPoll::poll(&self, &mut events.0, timeout)?;
    events.1 = 0;
    Ok(r)
  }
}


impl Registerable<MioPoll> for MioStream {
  fn register(&self, p: &MioPoll, t: Token, r: Ready) -> Result<bool> {
    match r {
      Ready::Readable => p.register(self, MioToken(t), MioReady::readable(), PollOpt::edge())?,
      Ready::Writable => p.register(self, MioToken(t), MioReady::writable(), PollOpt::edge())?,
    }

    Ok(true)
  }
  fn reregister(&self, p: &MioPoll, t: Token, r: Ready) -> Result<bool> {
    match r {
      Ready::Readable => p.reregister(self, MioToken(t), MioReady::readable(), PollOpt::edge())?,
      Ready::Writable => p.reregister(self, MioToken(t), MioReady::writable(), PollOpt::edge())?,
    }
    Ok(true)
  }

  fn deregister(&self, poll: &MioPoll) -> Result<()> {
    poll.deregister(self)?;
    Ok(())
  }
}



