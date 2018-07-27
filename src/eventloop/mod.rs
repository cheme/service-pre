

#[cfg(feature = "mio-transport")]
pub mod mio;

use ::error::{
  Result,
};

use std::time::Duration;

pub type Token = usize;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ready {
  Readable,
  Writable,
}
use std::net::TcpStream;

/// Registerable : register on a event loop of corresponding Poll.
/// Edge register kind.
pub trait Registerable<P> {
  /// registration on main io loop when possible (if not return false)
  fn register(&self, &P, Token, Ready) -> Result<bool>;
  /// async reregister
  fn reregister(&self, &P, Token, Ready) -> Result<bool>;

  fn deregister(&self, poll: &P) -> Result<()>;
}

pub trait TriggerReady: Clone {
  fn set_readiness(&self, ready: Ready) -> Result<()>;
}

pub trait Poll {
  type Events: Events;
  fn poll(&self, events: &mut Self::Events, timeout: Option<Duration>) -> Result<usize>;
}

pub trait Events: Iterator<Item = Event> {
  fn with_capacity(usize) -> Self;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Event {
  pub kind: Ready,
  pub token: Token,
}


impl<PO> Registerable<PO> for TcpStream {
  fn register(&self, _: &PO, _: Token, _: Ready) -> Result<bool> {
    Ok(false)
  }
  fn reregister(&self, _: &PO, _: Token, _: Ready) -> Result<bool> {
    Ok(false)
  }

  fn deregister(&self, _: &PO) -> Result<()> {
    Ok(())
  }
}

