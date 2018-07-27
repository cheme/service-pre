//! error for this crate, note that there is specific ErrorLevel which is in this crate to either
//! yield service, return error or panic.


//#[cfg(feature = "with-coroutine")]
//extern crate coroutine;

use std::sync::mpsc::{
  SendError,
  RecvError,
  TryRecvError,
};
use std::io::{
  Error as IoError,
  ErrorKind as IoErrorKind,
};
//  ChannelSendError,
//  ChannelRecvError,
 
error_chain! {
  // standard types
  types {
      Error, ErrorKind, ResultExt, Result;
  }

  links {
   //    Another(other_error::Error, other_error::ErrorKind) #[cfg(unix)];
  }

  foreign_links {
//        Fmt(::std::fmt::Error);
//    Io(::std::io::Error) #[cfg(unix)];
    ChannelRecv(RecvError);
    ChannelTryRecv(TryRecvError);
//    Coroutine(self::coroutine::Error) #[cfg(feature = "with-coroutine")];
  }

  errors {
    ChannelSend(t: String) {
      description("Channel send error")
      display("Channel send error: '{}'", t)
    }
    Bug(t: String) {
      description("A bug, should not happen")
      display("Bug: '{}'", t)
    }
    Panic(t: String) {
      description("A panic error")
      display("Panic: '{}'", t)
    }
    Io(err: IoError) {
      description(::std::error::Error::description(err))
      display("Io: '{:?}'", err)
    }
    Yield {
      description("Yield")
      display("Yield")
    }
    EndService {
      description("End Service")
      display("EndService")
    }
 
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum ErrorLevel {
  /// end program
  Panic,
  /// end current action but do not halt program (propagate or shut action (close connection or
  /// retry connect...)
  ShutAction,
  // go to next iter
  //Ignore,
  /// error is expected and should not end process
  Yield,
  // End service with expected error
  //ExpectedEnd,
}

impl Error {
  #[inline]
  pub fn level(&self) -> ErrorLevel {
    self.kind().level()
  }
}

impl ErrorKind {
  pub fn level(&self) -> ErrorLevel {
    match *self {
      ErrorKind::Bug(_) => ErrorLevel::Panic,
      ErrorKind::Panic(_) => ErrorLevel::Panic,
      ErrorKind::ChannelSend(_) => ErrorLevel::ShutAction,
      ErrorKind::ChannelRecv(_) => ErrorLevel::ShutAction,
      ErrorKind::ChannelTryRecv(_) => ErrorLevel::ShutAction,
      ErrorKind::Msg(_) => ErrorLevel::ShutAction,
      ErrorKind::Yield => ErrorLevel::Yield,
      ErrorKind::EndService => ErrorLevel::ShutAction,
      ErrorKind::Io(_) => ErrorLevel::ShutAction,
      // TODO transform to non exhaustive when stable
      ErrorKind::__Nonexhaustive{..} => ErrorLevel::Panic,
    }
  }
}



impl<T> From<SendError<T>> for Error {
  #[inline]
  fn from(e: SendError<T>) -> Error {
    ErrorKind::ChannelSend(e.to_string()).into()
  }
}

impl From<IoError> for Error {
  #[inline]
  fn from(e: IoError) -> Error {
    if let IoErrorKind::WouldBlock = e.kind() {
      ErrorKind::Yield.into()
    } else {
      ErrorKind::Io(e).into()
    }
  }
}

