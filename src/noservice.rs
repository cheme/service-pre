use ::{
  Service,
  SpawnerYield,
};

use ::error::{
  Result,
};

use std::marker::PhantomData;

pub struct NoService<I, O>(PhantomData<(I, O)>);

impl<CIN, COUT> NoService<CIN, COUT> {
  pub fn new() -> Self {
    NoService(PhantomData)
  }
}

impl<CIN, COUT> Clone for NoService<CIN, COUT> {
  fn clone(&self) -> Self {
    NoService(PhantomData)
  }
}

impl<CIN, COUT> Service for NoService<CIN, COUT> {
  type CommandIn = CIN;
  type CommandOut = COUT;

  fn call<S: SpawnerYield>(&mut self, _: Self::CommandIn, _: &mut S) -> Result<Self::CommandOut> {
    panic!("No service called : please use a dummy spawner with no service")
  }
}

