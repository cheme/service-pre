use ::{
  SpawnerYield,
  YieldReturn,
  SpawnUnyield,
  SpawnHandle,
  SpawnWeakUnyield,
  Service,
  SpawnSend,
  SpawnRecv,
  Spawner,
};



use error::{
  Result,
};


#[derive(Clone, Debug)]
pub struct NoSpawn;
#[derive(Clone, Debug)]
pub struct NoHandle;

#[derive(Clone)]
pub struct NoWeakUnyield;

impl SpawnUnyield for NoWeakUnyield {
  fn is_finished(&mut self) -> bool {
    unreachable!()
  }
  fn unyield(&mut self) -> Result<()> {
    unreachable!()
  }
}

impl<
    S: Service,
    D: SpawnSend<<S as Service>::CommandOut>,
    R: SpawnRecv<<S as Service>::CommandIn>,
  > Spawner<S, D, R> for NoSpawn
{
  type Handle = NoHandle;
  type Yield = NoYield;
  fn spawn(
    &mut self,
    _: S,
    _: D,
    _: Option<<S as Service>::CommandIn>,
    _: R,
    _: usize, // infinite if 0
  ) -> Result<Self::Handle> {
    Ok(NoHandle)
  }
}

impl SpawnUnyield for NoHandle {
  fn is_finished(&mut self) -> bool {
    false
  }
  fn unyield(&mut self) -> Result<()> {
    Ok(())
  }
}

impl<
    S: Service,
    D: SpawnSend<<S as Service>::CommandOut>,
    R: SpawnRecv<<S as Service>::CommandIn>,
  > SpawnHandle<S, D, R> for NoHandle
{
  fn unwrap_state(self) -> Result<(S, D, R, Result<()>)> {
    unreachable!("check is_finished before : No spawn is never finished");
  }
}

impl SpawnWeakUnyield for NoHandle {
  type WeakUnyield = NoHandle;
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    Some(NoHandle)
  }
}

#[derive(Clone)]
pub struct NoYield(pub YieldReturn);

impl SpawnerYield for NoYield {
  #[inline]
  fn spawn_yield(&mut self) -> YieldReturn {
    self.0
  }
  #[inline]
  fn opt_clone(&mut self) -> Option<Self> {
    Some(self.clone())
  }
}

