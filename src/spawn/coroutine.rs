//! Stackfull coroutine service

extern crate coroutine;

use error::{
  Result,
  ErrorKind,
  ErrorLevel,
};

use std::rc::Rc;
use std::cell::RefCell;


use self::coroutine::Error as CoroutError;
use self::coroutine::asymmetric::{Handle as CoRHandle, Coroutine as CoroutineC};
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

use ::spawn::void::{
  NoWeakUnyield,
};

pub struct Coroutine;

/// not type alias as will move from this crate
pub struct CoroutHandle<S, D, R>(Rc<RefCell<Option<(S, D, R, Result<()>)>>>, CoRHandle);



impl SpawnerYield for CoroutineC {
  #[inline]
  fn spawn_yield(&mut self) -> YieldReturn {
    self.yield_with(0);
    YieldReturn::Loop
  }
  #[inline]
  fn opt_clone(&mut self) -> Option<Self> {
    None
  }
}

impl<S, D, R> SpawnUnyield for CoroutHandle<S, D, R> {
  #[inline]
  fn is_finished(&mut self) -> bool {
    self.1.is_finished()
  }
  #[inline]
  fn unyield(&mut self) -> Result<()> {
    self
      .1
      .resume(0)
      .map_err(|e| {
        match e {
          CoroutError::Panicked => panic!("Spawned coroutine has panicked"),
          CoroutError::Panicking(c) => panic!(c),
        };
      })
      .unwrap();
    Ok(())
  }
}


impl<S, D, R> SpawnHandle<S, D, R> for CoroutHandle<S, D, R> {
  #[inline]
  fn unwrap_state(self) -> Result<(S, D, R, Result<()>)> {
    match self.0.replace(None) {
      Some(s) => Ok(s),
      None => Err(ErrorKind::Bug("Read an unfinished corouthandle".to_string()).into()),
    }
  }
}


impl<S, D, R> SpawnWeakUnyield for CoroutHandle<S, D, R> {
  type WeakUnyield = NoWeakUnyield;
  #[inline]
  fn get_weak_unyield(&self) -> Option<Self::WeakUnyield> {
    None
  }
}


impl<
    S: 'static + Service,
    D: 'static + SpawnSend<<S as Service>::CommandOut>,
    R: 'static + SpawnRecv<S::CommandIn>,
  > Spawner<S, D, R> for Coroutine
{
  type Handle = CoroutHandle<S, D, R>;
  //type Yield = CoroutineYield;
  type Yield = CoroutineC;
  fn spawn(
    &mut self,
    mut service: S,
    mut spawn_out: D,
    mut ocin: Option<<S as Service>::CommandIn>,
    mut recv: R,
    mut nb_loop: usize,
  ) -> Result<Self::Handle> {
    let rcs = Rc::new(RefCell::new(None));
    let rcs2 = rcs.clone();
    let co_handle = CoroutineC::spawn(move |corout, _| {
      move || -> Result<()> {
        let mut err = Ok(());
        let rcs = rcs;
        let mut yiel = corout;
        spawn_loop!(
          service,
          spawn_out,
          ocin,
          recv,
          nb_loop,
          yiel,
          err,
          Err(ErrorKind::Bug("Coroutine spawn service return would return when should loop".to_string()).into())
        );
        rcs.replace(Some((service, spawn_out, recv, err)));
        //replace(dest,Some((service,spawn_out,recv,err)));
        Ok(())
      }().unwrap();
      0
    });
    let mut handle = CoroutHandle(rcs2, co_handle);
    // start it
    handle.unyield()?;
    return Ok(handle);
  }
}

