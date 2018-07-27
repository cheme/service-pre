
extern crate futures_cpupool;
extern crate futures;

use self::futures_cpupool::CpuPool as FCpuPool;
use self::futures_cpupool::CpuFuture;
use self::futures::Async;
use std::mem::replace;
use std::thread;
use ::spawn::void::{
  NoWeakUnyield,
};
use self::futures::future::Future;
use self::futures::future::ok as okfuture;
use self::futures::future::err as errfuture;

use error::{
  Result,
  Error,
  ErrorKind,
  ErrorLevel,
};

use ::{
  YieldReturn,
  SpawnUnyield,
  SpawnHandle,
  SpawnWeakUnyield,
  Service,
  SpawnSend,
  SpawnRecv,
  Spawner,
  SpawnerYield,
};

/// TODO move in its own crate or under feature in mydht : cpupool dependency in mydht-base is
/// bad (future may be in api so fine)
/// This use CpuPool but not really the future abstraction
pub struct CpuPool(FCpuPool);
// TODO simpliest type with Result<() everywhere).
pub struct CpuPoolHandle<S, D, R>(
  CpuFuture<(S, D, R, Result<()>), Error>,
  Option<(S, D, R, Result<()>)>,
);

#[derive(Clone)]
pub struct CpuPoolYield;

impl<S: Send + 'static, D: Send + 'static, R: Send + 'static> SpawnUnyield
  for CpuPoolHandle<S, D, R>
{
  #[inline]
  fn is_finished(&mut self) -> bool {
    match self.0.poll() {
      Ok(Async::Ready(r)) => {
        self.1 = Some(r);
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
impl<S: Send + 'static, D: Send + 'static, R: Send + 'static> SpawnHandle<S, D, R>
  for CpuPoolHandle<S, D, R>
{
  #[inline]
  fn unwrap_state(mut self) -> Result<(S, D, R, Result<()>)> {
    if self.1.is_some() {
      let r = replace(&mut self.1, None);
      return Ok(r.unwrap());
    }
    self.0.wait()
  }
}
impl<S, D, R> SpawnWeakUnyield for CpuPoolHandle<S, D, R> {
  // weakHandle is doable but require some sync overhead (arc mutex)
  type WeakUnyield = NoWeakUnyield;
  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    None
  }
}

impl SpawnerYield for CpuPoolYield {
  #[inline]
  fn spawn_yield(&mut self) -> YieldReturn {
    thread::park(); // TODO check without (may block)
    YieldReturn::Loop
  }
  #[inline]
  fn opt_clone(&mut self) -> Option<Self> {
    Some(self.clone())
  }
}

impl<
    S: 'static + Send + Service,
    D: 'static + Send + SpawnSend<S::CommandOut>,
    R: 'static + Send + SpawnRecv<S::CommandIn>,
  > Spawner<S, D, R> for CpuPool
where
  S::CommandIn: Send,
{
  type Handle = CpuPoolHandle<S, D, R>;
  type Yield = CpuPoolYield;
  fn spawn(
    &mut self,
    mut service: S,
    mut spawn_out: D,
    mut ocin: Option<<S as Service>::CommandIn>,
    mut recv: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let future = self.0.spawn_fn(move || {
      match move || -> Result<(S, D, R, Result<()>)> {
        let mut err = Ok(());
        spawn_loop!(
          service,
          spawn_out,
          ocin,
          recv,
          nb_loop,
          CpuPoolYield,
          err,
          Err(ErrorKind::Bug("CpuPool spawn service return would return when should loop".to_string()).into())
        );
        Ok((service, spawn_out, recv, err))
      }() {
        Ok(r) => okfuture(r),
        Err(e) => errfuture(e),
      }
    });
    return Ok(CpuPoolHandle(future, None));
  }
}

