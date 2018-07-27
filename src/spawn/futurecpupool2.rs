
extern crate futures_cpupool;
extern crate futures;

use self::futures_cpupool::CpuPool as FCpuPool;
use self::futures_cpupool::CpuFuture;
use self::futures::Async;
use std::mem::replace;
use ::spawn::void::{
  NoWeakUnyield,
};
use self::futures::future::Future;
use self::futures::future::ok as okfuture;
use self::futures::future::err as errfuture;

use error::{
  Result,
  Error,
  ErrorLevel,
};

use ::{
  SpawnerYield,
  SpawnUnyield,
  SpawnHandle,
  SpawnWeakUnyield,
  Service,
  SpawnSend,
  SpawnRecv,
  Spawner,
};

use ::spawn::futurecpupool::CpuPoolYield;

/// This cpu pool use future internally TODO test it , the imediate impact upon CpuPool is that the
/// receiver for spawning is local (no need to be send)
pub struct CpuPoolFuture(FCpuPool);
pub struct CpuPoolHandleFuture<S, D, R>(
  CpuFuture<(S, D, Result<()>), Error>,
  Option<(S, D, Result<()>)>,
  R,
);

impl<S: Send + 'static + Service, R, D: 'static + Send + SpawnSend<S::CommandOut>> SpawnUnyield
  for CpuPoolHandleFuture<S, D, R>
{
  #[inline]
  fn is_finished(&mut self) -> bool {
    match self.0.poll() {
      Ok(Async::Ready(r)) => {
        self.1 = Some((r.0, r.1, Ok(())));
        true
      }
      Ok(Async::NotReady) => false,
      Err(_) => true, // the error will be reported throug call to unwrap_starte TODO log it before
    }
  }
  #[inline]
  fn unyield(&mut self) -> Result<()> {
    if !self.1.is_some() {
      self.is_finished();
    }
    Ok(())
  }
}

impl<S: Send + 'static + Service, R, D: 'static + Send + SpawnSend<S::CommandOut>>
  SpawnHandle<S, D, R> for CpuPoolHandleFuture<S, D, R>
{
  #[inline]
  fn unwrap_state(mut self) -> Result<(S, D, R, Result<()>)> {
    if self.1.is_some() {
      if let Some((s, d, res)) = replace(&mut self.1, None) {
        return Ok((s, d, self.2, res));
      } else {
        unreachable!()
      }
    }
    match self.0.wait() {
      Ok((s, d, r)) => Ok((s, d, self.2, r)),
      Err(e) => Err(e),
    }
  }
}
impl<S, R, D> SpawnWeakUnyield for CpuPoolHandleFuture<S, D, R> {
  // TODO could be implemented with arc mutex our content
  type WeakUnyield = NoWeakUnyield;
  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    None
  }
}

impl<
    S: 'static + Send + Service,
    D: 'static + Send + SpawnSend<S::CommandOut>,
    R: SpawnRecv<S::CommandIn>,
  > Spawner<S, D, R> for CpuPoolFuture
where
  S::CommandIn: Send,
{
  type Handle = CpuPoolHandleFuture<S, D, R>;
  type Yield = CpuPoolYield;
  fn spawn(
    &mut self,
    service: S,
    spawn_out: D,
    mut ocin: Option<<S as Service>::CommandIn>,
    mut recv: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let mut future = self.0.spawn(okfuture((service, spawn_out, Ok(()))));
    loop {
      match ocin {
        Some(cin) => {
          future = self
            .0
            .spawn(future.and_then(|(mut service, mut spawn_out, _)| {
              match service.call(cin, &mut CpuPoolYield) {
                Ok(r) => {
                  if D::CAN_SEND {
                    match spawn_out.send(r) {
                      Ok(_) => (),
                      Err(e) => return errfuture(e),
                    }
                  }
                  return okfuture((service, spawn_out, Ok(())));
                }
                Err(e) => if e.level() == ErrorLevel::Yield {
                  panic!("This should only yield loop, there is an issue with implementation");
                } else if e.level() == ErrorLevel::Panic {
                  panic!("In spawner cpufuture panic : {:?} ", e);
                } else {
                  return okfuture((service, spawn_out, Err(e)));
                  //                return errfuture((service,spawn_out,e))
                },
              };
            }));
          if nb_loop > 0 {
            nb_loop -= 1;
            if nb_loop == 0 {
              break;
            }
          }
        }
        None => {
          ocin = recv.recv()?;
          if ocin.is_none() {
            CpuPoolYield.spawn_yield();
          } else {
            continue;
          }
        }
      }
      ocin = None;
    }
    return Ok(CpuPoolHandleFuture(future, None, recv));
  }
}
