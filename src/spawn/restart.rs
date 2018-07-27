//! Restartable execution : this spawner run on the same thread

use error::{
  Result,
  ErrorLevel,
};

use ::{
  SpawnerYield,
  YieldReturn,
  SpawnUnyield,
  SpawnHandle,
  SpawnWeakUnyield,
  Service,
  ServiceRestartable,
  SpawnSend,
  SpawnRecv,
  Spawner,
};

use ::spawn::void::{
  NoYield,
  NoWeakUnyield,
};

use std::mem::replace;


/// For restartable service, running in the same thread, restart on service state
/// Notice that restart is done by spawning again : this kind of spawn can only be use at place
/// where it is known to restart
pub struct RestartOrError;


pub struct RestartSpawn<
  S: Service,
  D: SpawnSend<<S as Service>::CommandOut>,
  SP: Spawner<S, D, R>,
  R: SpawnRecv<S::CommandIn>,
> {
  spawner: SP,
  service: S,
  spawn_out: D,
  recv: R,
  nb_loop: usize,
}

pub enum RestartSameThread<
  S: Service,
  D: SpawnSend<<S as Service>::CommandOut>,
  SP: Spawner<S, D, R>,
  R: SpawnRecv<S::CommandIn>,
> {
  ToRestart(RestartSpawn<S, D, SP, R>),
  Ended((S, D, R, Result<()>)),
  // technical
  Empty,
}


/// Only for Restartable service
impl<
    S: Service + ServiceRestartable,
    D: SpawnSend<<S as Service>::CommandOut>,
    R: SpawnRecv<S::CommandIn>,
  > Spawner<S, D, R> for RestartOrError
{
  type Handle = RestartSameThread<S, D, Self, R>;
  type Yield = NoYield;
  fn spawn(
    &mut self,
    mut service: S,
    mut spawn_out: D,
    mut ocin: Option<<S as Service>::CommandIn>,
    mut recv: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let mut yiel = NoYield(YieldReturn::Return);
    let mut err = Ok(());
    spawn_loop_restartable!(
      service,
      spawn_out,
      ocin,
      recv,
      nb_loop,
      yiel,
      err,
      Ok(RestartSameThread::ToRestart(RestartSpawn {
        spawner: RestartOrError,
        service: service,
        spawn_out: spawn_out,
        recv: recv,
        nb_loop: nb_loop,
      }))
    );
    Ok(RestartSameThread::Ended((service, spawn_out, recv, err)))
  }
}


impl<
    S: Service + ServiceRestartable,
    D: SpawnSend<<S as Service>::CommandOut>,
    R: SpawnRecv<S::CommandIn>,
  > SpawnUnyield for RestartSameThread<S, D, RestartOrError, R>
{
  #[inline]
  fn is_finished(&mut self) -> bool {
    if let &mut RestartSameThread::Ended(_) = self {
      true
    } else {
      false
    }
  }
  #[inline]
  fn unyield(&mut self) -> Result<()> {
    if let &mut RestartSameThread::ToRestart(_) = self {
      let tr = replace(self, RestartSameThread::Empty);
      if let RestartSameThread::ToRestart(RestartSpawn {
        mut spawner,
        service,
        spawn_out,
        recv,
        nb_loop,
      }) = tr
      {
        let rs = spawner.spawn(service, spawn_out, None, recv, nb_loop)?;
        replace(self, rs);
      } else {
        unreachable!();
      }
    }
    Ok(())
  }
}
impl<
    S: Service + ServiceRestartable,
    D: SpawnSend<<S as Service>::CommandOut>,
    R: SpawnRecv<S::CommandIn>,
  > SpawnHandle<S, D, R> for RestartSameThread<S, D, RestartOrError, R>
{
  #[inline]
  fn unwrap_state(mut self) -> Result<(S, D, R, Result<()>)> {
    loop {
      match self {
        RestartSameThread::ToRestart(_) => {
          self.unyield()?;
        }
        RestartSameThread::Ended(t) => {
          return Ok(t);
        }
        _ => unreachable!(),
      }
    }
  }
}

impl<
    S: Service,
    D: SpawnSend<<S as Service>::CommandOut>,
    SP: Spawner<S, D, R>,
    R: SpawnRecv<S::CommandIn>,
  > SpawnWeakUnyield for RestartSameThread<S, D, SP, R>
{
  type WeakUnyield = NoWeakUnyield;
  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    None
  }
}

