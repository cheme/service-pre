//! asynch transport abstraction (TODO not sure if needed in this crate)
//!


use std::io::{
  Result,
  ErrorKind,
  Read,
  Write,
};

use super::{
  YieldReturn,
  SpawnerYield,
};


pub struct ReadYield<'a, R: 'a + Read, Y: 'a + SpawnerYield>(pub &'a mut R, pub &'a mut Y);

pub struct WriteYield<'a, W: 'a + Write, Y: 'a + SpawnerYield>(pub &'a mut W, pub &'a mut Y);

impl<'a, R: Read, Y: SpawnerYield> Read for ReadYield<'a, R, Y> {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    loop {
      match self.0.read(buf) {
        Ok(r) => {
          if r == 0 {
            continue;
          }
          return Ok(r);
        }
        Err(e) => if let ErrorKind::WouldBlock = e.kind() {
          //println!("-> a read yield {:?}", thread::current().id());
          match self.1.spawn_yield() {
            YieldReturn::Return => return Err(e),
            YieldReturn::Loop => (),
          }
        } else {
          return Err(e);
        },
      }
    }
  }
  /// Variant of default read_exact where we block on read returning 0 instead of returning an
  /// error (for a transport stream it makes more sense)
  fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
    while !buf.is_empty() {
      match self.read(buf) {
        Ok(0) => (),
        Ok(n) => {
          let tmp = buf;
          buf = &mut tmp[n..];
        },
        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
        Err(e) => return Err(e),
      }
    }
    Ok(())
  }
}

impl<'a, W: Write, Y: SpawnerYield> Write for WriteYield<'a, W, Y> {
  fn write(&mut self, buf: &[u8]) -> Result<usize> {
    loop {
      match self.0.write(buf) {
        Ok(r) => return Ok(r),
        Err(e) => if let ErrorKind::WouldBlock = e.kind() {
          //println!("-> a write yield {:?}", thread::current().id());
          match self.1.spawn_yield() {
            YieldReturn::Return => return Err(e),
            YieldReturn::Loop => (),
          }
        } else {
          return Err(e);
        },
      }
    }
  }

  fn flush(&mut self) -> Result<()> {
    loop {
      match self.0.flush() {
        Ok(r) => return Ok(r),
        Err(e) => if let ErrorKind::WouldBlock = e.kind() {
          match self.1.spawn_yield() {
            YieldReturn::Return => return Err(e),
            YieldReturn::Loop => (),
          }
        } else {
          return Err(e);
        },
      }
    }
  }

}




