use error::{
  Result,
  ErrorKind,
  ErrorLevel,
};


use ::{
  Service,
  SpawnSend,
  SpawnRecv,
  Spawner,
  YieldReturn,
  SpawnUnyield,
  SpawnWeakUnyield,
  SpawnHandle,
  SpawnerYield,
};

use super::void::{
  NoYield,
  NoWeakUnyield,
};


#[derive(Clone)]
/// Should not be use, except for very specific case or debugging.
/// It blocks the main loop during process (run on the same thread).
//pub struct Blocker<S : Service>(PhantomData<S>);
pub struct Blocker;

pub struct BlockingSameThread<S, D, R>((S, D, R, Result<()>));

impl<S: Service, D: SpawnSend<<S as Service>::CommandOut>, R: SpawnRecv<S::CommandIn>>
  Spawner<S, D, R> for Blocker
{
  type Handle = BlockingSameThread<S, D, R>;
  type Yield = NoYield;
  fn spawn(
    &mut self,
    mut service: S,
    mut spawn_out: D,
    mut ocin: Option<<S as Service>::CommandIn>,
    mut r: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let mut yiel = NoYield(YieldReturn::Loop);
    let mut err = Ok(());
    spawn_loop!(
      service,
      spawn_out,
      ocin,
      r,
      nb_loop,
      yiel,
      err,
      Err(ErrorKind::Bug("Blocking spawn service return would return when should loop".to_string()).into())
    );
    return Ok(BlockingSameThread((service, spawn_out, r, err)));
  }
}

impl<S, D, R> SpawnUnyield for BlockingSameThread<S, D, R> {
  #[inline]
  fn is_finished(&mut self) -> bool {
    true // if called it must be finished
  }
  #[inline]
  fn unyield(&mut self) -> Result<()> {
    Ok(())
  }
}
impl<S, D, R> SpawnHandle<S, D, R> for BlockingSameThread<S, D, R> {
  #[inline]
  fn unwrap_state(self) -> Result<(S, D, R, Result<()>)> {
    Ok(self.0)
  }
}
impl<S, D, R> SpawnWeakUnyield for BlockingSameThread<S, D, R> {
  type WeakUnyield = NoWeakUnyield;
  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    None
  }
}
