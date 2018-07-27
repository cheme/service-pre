

use ::error::{
  Result,
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

use parking_lot::{
  Mutex,
};
use std::sync::{
  Arc,
  atomic::{
    Ordering,
    AtomicBool,
  },
};

use std::thread::{
  self,
  JoinHandle,
};


use std::mem::replace;

pub struct ThreadBlock;

#[derive(Clone)]
pub struct ThreadYieldBlock;
pub struct ThreadHandleBlock<S, D, R> {
  return_state: Arc<Mutex<Option<(S, D, R, Result<()>)>>>,
  th_join: JoinHandle<Result<()>>,
  finished: Arc<AtomicBool>,
}

pub struct ThreadHandleBlockWeak {
  finished: Arc<AtomicBool>,
}

impl Clone for ThreadHandleBlockWeak {
  fn clone(&self) -> Self {
    ThreadHandleBlockWeak {
      finished: self.finished.clone(),
    }
  }
}


impl SpawnUnyield for ThreadHandleBlockWeak {
  #[inline]
  fn is_finished(&mut self) -> bool {
    self.finished.load(Ordering::Relaxed)
  }
  #[inline]
  fn unyield(&mut self) -> Result<()> {
    Ok(())
  }
}

impl<S, D, R> SpawnUnyield for ThreadHandleBlock<S, D, R> {
  #[inline]
  fn is_finished(&mut self) -> bool {
    self.finished.load(Ordering::Relaxed)
  }
  #[inline]
  fn unyield(&mut self) -> Result<()> {
    Ok(())
  }
}

impl<S, D, R> SpawnHandle<S, D, R> for ThreadHandleBlock<S, D, R> {
  #[inline]
  fn unwrap_state(self) -> Result<(S, D, R, Result<()>)> {
    let mut mlock = self.return_state.lock();
    if mlock.is_some() {
      //let ost = mutex.into_inner();
      //self.2.store(true,Ordering::Relaxed);
      let ost = replace(&mut (*mlock), None);
      Ok(ost.unwrap())
    } else {
      Err(ErrorKind::Bug("unwrap state on unfinished thread".to_string()).into())
    }
  }
}



impl<S, D, R> SpawnWeakUnyield for ThreadHandleBlock<S, D, R> {
  type WeakUnyield = ThreadHandleBlockWeak;

  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    Some(ThreadHandleBlockWeak {
      finished: self.finished.clone(),
    })
  }
}

impl SpawnerYield for ThreadYieldBlock {
  #[inline]
  fn spawn_yield(&mut self) -> YieldReturn {
    thread::yield_now();
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
  > Spawner<S, D, R> for ThreadBlock
where
  S::CommandIn: Send,
{
  type Handle = ThreadHandleBlock<S, D, R>;
  type Yield = ThreadYieldBlock;
  fn spawn(
    &mut self,
    mut service: S,
    mut spawn_out: D,
    mut ocin: Option<<S as Service>::CommandIn>,
    mut recv: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let finished = (Arc::new(Mutex::new(None)), Arc::new(AtomicBool::new(false)));
    let finished2 = finished.clone();
    let join_handle = thread::Builder::new().spawn(move || {
      let mut err = Ok(());
      spawn_loop!(
        service,
        spawn_out,
        ocin,
        recv,
        nb_loop,
        ThreadYieldBlock,
        err,
        Err(ErrorKind::Bug("Thread block spawn service return would return when should loop".to_string()).into())
      );
      let mut data = finished.0.lock();
      *data = Some((service, spawn_out, recv, err));
      finished.1.store(true, Ordering::Relaxed);
      Ok(())
    })?;
    return Ok(ThreadHandleBlock {
      return_state: finished2.0,
      th_join: join_handle,
      finished: finished2.1,
    });
  }
}

