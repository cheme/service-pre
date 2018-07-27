//! Spawn implementations

use ::{
  SpawnerYield,
  YieldReturn,
};

macro_rules! spawn_loop {
  (
    $service:ident,
    $spawn_out:ident,
    $ocin:ident,
    $r:ident,
    $nb_loop:ident,
    $yield_build:ident,
    $result:ident,
    $return_build:expr
  ) => {
    spawn_loop_inner!(
      $service,
      $spawn_out,
      $ocin,
      $r,
      $nb_loop,
      $yield_build,
      $result,
      $return_build,
      { None }
    )
  };
}

macro_rules! spawn_loop_restartable {
  (
    $service:ident,
    $spawn_out:ident,
    $ocin:ident,
    $r:ident,
    $nb_loop:ident,
    $yield_build:ident,
    $result:ident,
    $return_build:expr
  ) => {
    spawn_loop_inner!(
      $service,
      $spawn_out,
      $ocin,
      $r,
      $nb_loop,
      $yield_build,
      $result,
      $return_build,
      {
        match $service.restart(&mut $yield_build) {
          Ok(Some(r)) => Some(Ok(r)),
          _ => None,
        }
      }
    )
  };
}

// TODO make it a function returning result of type parameter
macro_rules! spawn_loop_inner {
  (
    $service:ident,
    $spawn_out:ident,
    $ocin:ident,
    $r:ident,
    $nb_loop:ident,
    $yield_build:ident,
    $result:ident,
    $return_build:expr,
    $restart_code:expr
  ) => {
    let mut ores: Option<Result<_>> = None;
    loop {
      if let Some(res) = ores {
        match res {
          Ok(r) => {
            if D::CAN_SEND {
              $spawn_out.send(r)?;
            }
          },
          Err(e) => if e.level() == ErrorLevel::Yield {
            // suspend with YieldReturn
            return $return_build;
          } else if e.level() == ErrorLevel::Panic {
            error!("      !!!panicking{:?}", e);
            println!("      !!!panicking{:?}", e);
            panic!("In spawner : {:?}", e);
          } else {
            error!("      !!!error ending service{:?}", e);
            println!("      !!!error ending service{:?}", e);
            $result = Err(e);
            break;
          },
        }
        if $nb_loop > 0 {
          $nb_loop -= 1;
          if $nb_loop == 0 {
            break;
          }
        }
      }
      ores = None;
      match $ocin {
        Some(cin) => {
          ores = Some($service.call(cin, &mut $yield_build));
        }
        None => {
          $ocin = $r.recv()?;
          if $ocin.is_none() {
            ores = $restart_code;
            if ores.is_some() {
              continue;
            }
            if let YieldReturn::Return = $yield_build.spawn_yield() {
              return $return_build;
            } else {
              continue;
            }
          } else {
            continue;
          }
        }
      }
      $ocin = None;
    }
  };
}


pub mod blocking;
pub mod restart;
pub mod void;
pub mod threadblock;
pub mod threadpark;
pub mod futurecpupool;
pub mod futurecpupool2;

#[cfg(feature = "with-coroutine")]
pub mod coroutine;




impl<'a, A: SpawnerYield> SpawnerYield for &'a mut A {
  #[inline]
  fn spawn_yield(&mut self) -> YieldReturn {
    (*self).spawn_yield()
  }
  #[inline]
  fn opt_clone(&mut self) -> Option<Self> {
    //    self.opt_clone()
    None
  }
}

