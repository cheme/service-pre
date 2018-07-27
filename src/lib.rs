
//! Service trait : this trait simply translate the idea of a query and an asynch reply.
//!
//! Service Failure is returend as result to the spawner : the inner service process returning `Result` is here
//! for this purpose only. It should respawn or close.
//!
//! Command reply and error are send to another spawnhandle (as a Command).
//!
//! Spawner can loop (for instance a receive stream on an established transport connection) or stop and restart using `State parameter` and command remaining. Or combine both approach, reducing cost of state usage and avoiding stalled loop.
//!
//! Reply and Error are inherent to the command used
//! Service can be spawn using different strategies thrus the Spawn trait.
//!
//! For KVStore state would be initialization function (not function once as must be return ), or
//! builder TODO see kvstore refactoring
//! For peer reader, it would be its stream.
//!


#![recursion_limit = "1024"]

#[macro_use]
extern crate log;

#[macro_use]
extern crate immut_send;

#[macro_use]
extern crate error_chain;

#[cfg(feature = "with-coroutine")]
extern crate coroutine;
extern crate parking_lot;
extern crate rust_proto;
#[cfg(feature = "mio-transport")]
extern crate mio;


/*use std::sync::atomic::{
  AtomicBool,
  AtomicUsize,
  Ordering,
};*/
use error::{
  Result,
};


pub mod error;
pub mod spawn;
pub mod eventloop;
pub mod noservice;
pub mod transport;
pub mod channels;






/// TODO duration before restart (in conjonction with nb loop)
/// The service struct can have an inner State (mutable in call).
pub trait Service {
  /// If state contains all info needed to restart the service at the point it return a mydhterror
  /// of level ignore (wouldblock), set this to true; most of the time it should be set to false
  /// (safer).
  type CommandIn;
  type CommandOut;

  /// Allways Return a CommandOut, Error should be a command TODO consider returning error and
  /// having another SpawnSend to manage it (to avoid composing SpawnSend in enum).
  /// Returning Error for async would block or error which should shut the spawner (the main loop).
  /// If no command is
  /// For restartable service no command in can be send, if no command in is send and it is not
  /// restartable, an error is returned (if input is expected) : use const for testing before call
  fn call<S: SpawnerYield>(
    &mut self,
    req: Self::CommandIn,
    async_yield: &mut S,
  ) -> Result<Self::CommandOut>;
}

/// marker trait for service allowing restart (yield should return `YieldReturn::Return`).
/// if ignore level error (eg async wouldblock) is send should we report as a failure or ignore and restart with
/// state later (for instance if a register async stream token is available again in a epoll).
/// This depend on the inner implementation : if it could suspend on error and restart latter
/// with same state parameter.
pub trait ServiceRestartable: Service {
  /// same as standard service call without the need to have a request.
  ///
  /// If there is nothing to restart the service can reply without a `CommandOut`.
  ///
  /// Note that a restartable service must manage its suspended state also in standard service
  /// `call`
  /// (`restart` is only call if `spawn` (presumably after a `unyield`) do not have a request.
  ///
  /// For restartable service which are restartable because they do not yield, this function will
  /// simply return Ok(None), that is its default implementation.
  fn restart<S: SpawnerYield>(&mut self, _async_yield: &mut S) -> Result<Option<Self::CommandOut>> {
    Ok(None)
  }
}



///
/// SpawnHandle allow management of service from outside.
/// SpawnerYield allows inner service management, and only for yield (others action could be
/// managed by Spawner implementation (catching errors and of course ending spawn).
///
/// With parallel execution of spawn and caller, SpawnHandle and SpawnerYield must be synchronized.
/// A typical concurrency issue being a yield call and an unyield happenning during this yield
/// execution, for instance with an asynch read on a stream : a read would block leading to
/// unyield() call, during unyield the read stream becomes available again and unyield is called :
/// yield being not finished unyield may be seen as useless and after yield occurs it could be
/// stuck. An atomic state is therefore required, to skip a yield (returning YieldSpawn::Loop
/// even if it is normally a YieldSpawn::Return) is fine but skipping a unyield is bad.
/// Therefore unyield may add a atomic yield skip to its call when this kind of behaviour is expected.
/// Skiping a yield with loop returning being not an issue.
/// Same thing for channel read and subsequent yield.
/// Yield prototype does not include blocking call or action so the skip once is the preffered synch
/// mecanism (cost one loop : a second channel read call or write/read stream call), no protected
/// section.
pub trait Spawner<
  S: Service,
  D: SpawnSend<<S as Service>::CommandOut>,
  R: SpawnRecv<<S as Service>::CommandIn>,
>
{
  type Handle: SpawnHandle<S, D, R>;
  type Yield: SpawnerYield;
  /// use 0 as nb loop if service should loop forever (for instance on a read receiver or a kvstore
  /// service)
  /// TODO nb_loop is not enought to manage spawn, some optional duration could be usefull (that
  /// could also be polled out of the spawner : with a specific suspend command
  fn spawn(
    &mut self,
    service: S,
    spawn_out: D,
    //    state : Option<<S as Service>::State>,
    Option<<S as Service>::CommandIn>,
    rec: R,
    nb_loop: usize, // infinite if 0
  ) -> Result<Self::Handle>;
}


/// Handle use to send command to get back state
/// State in the handle is simply the Service struct
pub trait SpawnHandle<Service, Sen, Recv>: SpawnUnyield + SpawnWeakUnyield {
  /// if finished (implementation should panic if methode call if not finished), error management
  /// through last result and restart service through 3 firt items.
  /// Error are only technical : service error should send errormessge in sender.
  fn unwrap_state(self) -> Result<(Service, Sen, Recv, Result<()>)>;

  // TODO add a kill command and maybe a yield command
  //
  // TODO add a get_technical_error command : right now we do not know if we should restart on
  // finished or drop it!!!! -> plus solve the question of handling panic and error management
  // This goes with is_finished redesign : return state
}


pub trait SpawnUnyield {
  /// self is mut as most of the time this function is used in context where unwrap_state should be
  /// use so it allows more possibility for implementating
  ///
  /// TODO error management for is_finished plus is_finished in threads implementation is currently
  /// extra racy -> can receive message while finishing  : not a huge issue as the receiver remains
  /// into spawnhandle : after testing is_finished to true the receiver if not empty could be
  /// reuse. -> TODO some logging in case it is relevant and requires a synch.
  ///
  /// is_finished is not included in unwrap_state because it should be optimized by implementation
  /// (called frequently return single bool).
  fn is_finished(&mut self) -> bool;
  /// unyield implementation must take account of the fact that it is called frequently even if the
  /// spawn is not yield (no is_yield function here it is implementation dependant).
  /// For parrallel spawner (threading), unyield should position a skip atomic to true in case
  /// where it does not actually unyield.
  fn unyield(&mut self) -> Result<()>;
}

// TODO fuse in spawnunyield (check constraints)
pub trait SpawnWeakUnyield {
  type WeakUnyield: SpawnUnyield + Send + Clone;
  /// not all implementation allow weakhandle
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield>;
}

/// manages asynch call by possibly yielding process (yield a coroutine if same thread, park or
/// yield a thread, do nothing (block)
pub trait SpawnerYield: Sized {
  /// For parrallel spawner (threading), it is ok to skip yield once in case of possible racy unyield
  /// (unyield may not be racy : unyield on spawnchannel instead of asynch stream or others), see
  /// threadpark implementation
  /// At the time no SpawnerYield return both variant of YieldReturn and it could be a constant
  /// (use of both mode is spawner is not very likely but possible).
  fn spawn_yield(&mut self) -> YieldReturn;
  /// spawner yield should be clone but currently no satisfying implementation for Coroutine
  fn opt_clone(&mut self) -> Option<Self>;
}

/// For parrallel spawner (threading), it is ok to skip yield once in case of possible racy unyield
#[derive(Clone, Copy)]
pub enum YieldReturn {
  /// end spawn, transmit state to handle. Some Service may be restartable if enough info in state.
  /// This return an ignore Mydht level error.
  Return,
  /// retry at the same point (stack should be the same)
  Loop,
}


/// Send/Recv service Builder : TODO rename to Service Channel
pub trait SpawnChannel<Command> {
  type WeakSend: SpawnSend<Command> + Send + Clone;
  type Send: SpawnSend<Command> + Clone;
  type Recv: SpawnRecv<Command>;
  fn new(&mut self) -> Result<(Self::Send, Self::Recv)>;
  // SRef trait on send could be use but SToRef is useless : unneeded here
  fn get_weak_send(&Self::Send) -> Option<Self::WeakSend>;
}

/// send command to spawn, this is a blocking send currently : should evolve to non blocking.
/// TODO rename to ServiceSend
pub trait SpawnSend<Command>: Sized {
  /// if send is disable set to false, this can be test before calling `send`
  const CAN_SEND: bool;
  /// mut on self is currently not needed by impl
  fn send(&mut self, Command) -> Result<()>;
}

/// send command to spawn, it is non blocking (None returned by recv on not ready) : cf macro
/// spawn_loop or others implementation
/// TODO consider changing to Iterator of Result<Command>, that way we could for instance use repeat instead
/// of default recv or others... Yet hving Result<Options<C>> instead of Option<Result<C>> seems
/// better
/// TODO rename to ServiceRecv
pub trait SpawnRecv<Command>: Sized {
  /// mut on self is currently not needed by impl
  fn recv(&mut self) -> Result<Option<Command>>;
  //  fn new_sender(&mut self) -> Result<Self::SpawnSend>;
}



