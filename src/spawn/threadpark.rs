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

use std::sync::{
  Arc,
  atomic::{
    Ordering,
    AtomicBool,
  },
};

use parking_lot::{
  Mutex,
  Condvar,
};
use std::thread::{
  self,
  JoinHandle,
};

use error::{
  Result,
  ErrorKind,
  ErrorLevel,
};

use immut_send::{
  SRef,
  SToRef,
};

use std::mem::replace;

/// park thread on yield, unpark with Handle.
/// Std thread park not used for parking_lot mutex usage, but implementation
/// is the same.
/// It must be notice that after a call to unyield, yield will skip (avoid possible lock).
pub struct ThreadPark;
/// variant of thread park using command and channel as Ref to obtain send versio
pub struct ThreadParkRef;



#[derive(Clone)]
pub struct ThreadHandleParkWeak {
  park_sync: Arc<(Mutex<bool>, Condvar)>,
  finished: Arc<AtomicBool>,
}

pub struct ThreadHandlePark<S, D, R> {
  return_state: Arc<Mutex<Option<(S, D, R, Result<()>)>>>,
  th_join: JoinHandle<Result<()>>,
  park_sync: Arc<(Mutex<bool>, Condvar)>,
  finished: Arc<AtomicBool>,
}

pub struct ThreadHandleParkRef<S: SRef, D: SRef, R: SRef> {
  return_state: Arc<Mutex<Option<(S::Send, D::Send, R::Send, Result<()>)>>>,
  th_join: JoinHandle<Result<()>>,
  park_sync: Arc<(Mutex<bool>, Condvar)>,
  finished: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct ThreadYieldPark {
  park_sync: Arc<(Mutex<bool>, Condvar)>,
}

macro_rules! thread_parkunyield(() => (
  #[inline]
  fn is_finished(&mut self) -> bool {
    self.finished.load(Ordering::Relaxed)
  }
  #[inline]
  fn unyield(&mut self) -> Result<()> {
    let mut guard = (self.park_sync).0.lock();
    if !*guard {
      *guard = true;
      (self.park_sync).1.notify_one();
    }
    Ok(())
  }

));

impl SpawnUnyield for ThreadHandleParkWeak {
  thread_parkunyield!();
}

impl<S: SRef, D: SRef, R: SRef> SpawnUnyield for ThreadHandleParkRef<S, D, R> {
  thread_parkunyield!();
}

impl<S, D, R> SpawnUnyield for ThreadHandlePark<S, D, R> {
  thread_parkunyield!();
}

impl<S: SRef, D: SRef, R: SRef> SpawnHandle<S, D, R> for ThreadHandleParkRef<S, D, R> {
  #[inline]
  fn unwrap_state(self) -> Result<(S, D, R, Result<()>)> {
    let mut mlock = self.return_state.lock();
    if mlock.is_some() {
      //let ost = mutex.into_inner();
      let ost = replace(&mut (*mlock), None);
      let (s, d, r, rr) = ost.unwrap();
      Ok((s.to_ref(), d.to_ref(), r.to_ref(), rr))
    } else {
      Err(ErrorKind::Bug("unwrap state on unfinished thread".to_string()).into())
    }
  }
}

impl<S: SRef, D: SRef, R: SRef> SpawnWeakUnyield for ThreadHandleParkRef<S, D, R> {
  //  type WeakUnyield = NoWeakUnyield;
  type WeakUnyield = ThreadHandleParkWeak;

  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    Some(ThreadHandleParkWeak {
      park_sync: self.park_sync.clone(),
      finished: self.finished.clone(),
    })
  }
}

impl<S, D, R> SpawnHandle<S, D, R> for ThreadHandlePark<S, D, R> {
  #[inline]
  fn unwrap_state(self) -> Result<(S, D, R, Result<()>)> {
    let mut mlock = self.return_state.lock();
    if mlock.is_some() {
      //let ost = mutex.into_inner();
      let ost = replace(&mut (*mlock), None);
      Ok(ost.unwrap())
    } else {
      Err(ErrorKind::Bug("unwrap state on unfinished thread".to_string()).into())
    }
  }
}

impl<S, D, R> SpawnWeakUnyield for ThreadHandlePark<S, D, R> {
  type WeakUnyield = ThreadHandleParkWeak;

  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    Some(ThreadHandleParkWeak {
      park_sync: self.park_sync.clone(),
      finished: self.finished.clone(),
    })
  }
}

impl SpawnerYield for ThreadYieldPark {
  #[inline]
  fn spawn_yield(&mut self) -> YieldReturn {
    //if self.0.compare_and_swap(true, false, Ordering::Acquire) != true {
    //  if let Err(status) = (self.0).0.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed) {
    // } else {
    // status 0, we yield only if running, in other state we simply loop and will yield at next
    // call TODO could optimize by using mutex directly

    // park (thread::park();)
    {
      let mut guard = (self.park_sync).0.lock();
      //      (self.0).0.store(2,Ordering::Release);
      while !*guard {
        (self.park_sync).1.wait(&mut guard);
        //guard = (self.0).1.wait(guard);
      }
      *guard = false;
    }
    //}
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
  > Spawner<S, D, R> for ThreadPark
where
  S::CommandIn: Send,
{
  type Handle = ThreadHandlePark<S, D, R>;
  type Yield = ThreadYieldPark;
  fn spawn(
    &mut self,
    mut service: S,
    mut spawn_out: D,
    mut ocin: Option<<S as Service>::CommandIn>,
    mut recv: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let skip = Arc::new((Mutex::new(false), Condvar::new()));
    let skip2 = skip.clone();
    let finished = (Arc::new(Mutex::new(None)), Arc::new(AtomicBool::new(false)));
    let finished2 = finished.clone();
    let join_handle = thread::Builder::new().spawn(move || {
      let mut y = ThreadYieldPark { park_sync: skip2 };

      let mut err = Ok(());
      spawn_loop!(
        service,
        spawn_out,
        ocin,
        recv,
        nb_loop,
        y,
        err,
        Err(ErrorKind::Bug("Thread park spawn service return would return when should loop".to_string()).into())
      );
      let mut data = finished.0.lock();
      *data = Some((service, spawn_out, recv, err));
      finished.1.store(true, Ordering::Relaxed);
      Ok(())
    })?;
    return Ok(ThreadHandlePark {
      return_state: finished2.0,
      th_join: join_handle,
      park_sync: skip,
      finished: finished2.1,
    });
  }
}



impl<
    S: 'static + Service + SRef,
    D: SRef + 'static + SpawnSend<S::CommandOut>,
    R: 'static + SRef + SpawnRecv<S::CommandIn>,
  > Spawner<S, D, R> for ThreadParkRef
where
  <S as Service>::CommandIn: SRef,
{
  /*impl<S : 'static + Send + Service, D : 'static + Send + SpawnSend<S::CommandOut>, R : 'static + Send + SpawnRecv<
<S::CommandIn as Ref<S::CommandIn>>::ToRef<S::CommandIn>
>> 
  Spawner<S,D,R> for ThreadParkRef
  where S::CommandIn : Ref<S::CommandIn>
{*/
  type Handle = ThreadHandleParkRef<S, D, R>;
  type Yield = ThreadYieldPark;
  fn spawn(
    &mut self,
    service: S,
    spawn_out: D,
    ocin: Option<<S as Service>::CommandIn>,
    recv: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let skip = Arc::new((Mutex::new(false), Condvar::new()));
    let skip2 = skip.clone();
    let finished = (Arc::new(Mutex::new(None)), Arc::new(AtomicBool::new(false)));
    let finished2 = finished.clone();

    let service_ref = service.get_sendable();
    let spawn_out_s = spawn_out.get_sendable();
    let recv_s = recv.get_sendable();
    let ocins = ocin.map(|cin| cin.get_sendable());
    let join_handle = thread::Builder::new().spawn(move || {
      let mut service = service_ref.to_ref();
      let mut spawn_out = spawn_out_s.to_ref();
      let mut recv = recv_s.to_ref();
      let mut y = ThreadYieldPark { park_sync: skip2 };
      let mut err = Ok(());
      let mut ocin = ocins.map(|cin| cin.to_ref());
      spawn_loop!(
        service,
        spawn_out,
        ocin,
        recv,
        nb_loop,
        y,
        err,
        Err(ErrorKind::Bug("Thread park spawn service return would return when should loop".to_string()).into())
      );
      let mut data = finished.0.lock();
      *data = Some((
        service.get_sendable(),
        spawn_out.get_sendable(),
        recv.get_sendable(),
        err,
      ));
      finished.1.store(true, Ordering::Relaxed);
      Ok(())
    })?;
    return Ok(ThreadHandleParkRef {
      return_state: finished2.0,
      th_join: join_handle,
      park_sync: skip,
      finished: finished2.1,
    });
  }
}

