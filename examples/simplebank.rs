use ::futures::{
  channel::mpsc::{UnboundedReceiver, UnboundedSender},
  executor, try_join
};
#[allow(unused_imports)]
use ::rumpsteak::{
  channel::Bidirectional, session, Branch, End, Message, Receive, Role, Roles, Select, Send, 
  try_session
};

// why does it not generate this for us automatically...?
use std::{error::Error, fmt, cmp};

type Channel = Bidirectional<UnboundedSender<Label>, UnboundedReceiver<Label>>;

#[derive(Roles)]
#[allow(dead_code)]
struct Roles {
  c: C,
  b: B,
}

#[derive(Role)]
#[message(Label)]
struct C {
  #[route(B)]
  b: Channel,
}

#[derive(Role)]
#[message(Label)]
struct B {
  #[route(C)]
  c: Channel,
}

#[derive(Message)]
#[derive(Clone, Copy)]
#[derive(Debug)]
enum Label {
  Transfer(Transfer),
  Ok(Okay),
  Ko(Ko),
}

#[derive(Clone, Copy)]
#[derive(Debug)]
struct Transfer(PL1);

#[derive(Clone, Copy)]
#[derive(Debug)]
struct Okay(PL2);

#[derive(Clone, Copy)]
#[derive(Debug)]
struct Ko(PL3);

#[session]
type SimpleBankC = Send<B, Transfer, Branch<B, SimpleBankC1>>;

#[session]
enum SimpleBankC1 {
  Ko(Ko, End),
  Ok(Okay, End),
}

#[session]
type SimpleBankB = Receive<C, Transfer, Select<C, SimpleBankB1>>;

#[session]
enum SimpleBankB1 {
  Ko(Ko, End),
  Ok(Okay, End),
}

// -- [Generated above, written below] ----------------------------------------

#[derive(Clone, Copy)]
#[derive(Debug)]
struct PL1 {
  accountSrc : u32, // Nat
  accountTgt : u32,
  amount : u32,
}
#[derive(Clone, Copy)]
#[derive(Debug)]
struct PL2 {
  value : bool
}
#[derive(Clone, Copy)]
#[derive(Debug)]
struct PL3 {
  value : bool
}

impl fmt::Display for C {
  fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "rC")
  }
}

impl fmt::Display for B {
  fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "rB")
  }
}

impl fmt::Display for Label {
  fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "label: {}", self.to_string())
  }
}

impl fmt::Display for Transfer {
  fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "lTransfer({})", self.0)
  }
}

impl fmt::Display for Okay {
  fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "lOkay({})", self.0)
  }
}

impl fmt::Display for Ko {
  fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "lKo({})", self.0)
  }
}

impl fmt::Display for PL1 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PL1( act1 = {}, act2 = {}, amt = {} )", self.accountSrc, self.accountTgt, self.amount)
  }
}

impl fmt::Display for PL2 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PL2( value = {} )", self.value)
  }
}

impl fmt::Display for PL3 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PL3( value = {} )", self.value)
  }
}

async fn c(role : &mut C) -> Result<(), Box<dyn Error>> {
  try_session(role, |s : SimpleBankC<'_, _>|  async {
    let m : PL1 = PL1 { accountSrc: 1, accountTgt: 2, amount: 10 };
    let s = s.send(Transfer(m)).await?;
    match s.branch().await? {
      SimpleBankC1::Ok(x, end) => {
        Result::Ok(((), end))
      }
      SimpleBankC1::Ko(y, end) => {
        Result::Ok(((), end))
      }
    }
  }).await
}

fn b1(x : PL1, acts : Vec<(u32,u32)>) -> Result<Vec<(u32,u32)>, UpdateValueErr<u32,u32>> {
  let f = |y : u32| -> u32 {y - x.amount};
  let g = |y : u32| -> u32 {y + x.amount};
  let acts = updateValue(x.accountSrc, acts, &f)?;
  updateValue(x.accountTgt, acts, &g)
}

async fn b(role : &mut B, acts : Vec<(u32,u32)>) -> Result<(), Box<dyn Error>> {
  try_session(role, |s : SimpleBankB<'_, _>| async {
    let (Transfer(x), s) = s.receive().await?;
    if isKey(x.accountSrc, &acts)
       && isKey(x.accountTgt, &acts)
       && lookupKey(x.accountSrc, &acts) >= Some(&x.amount) {
      match b1(x, acts) {
        Ok(acts) => {
          println!("acts: {:#?}", &acts);
          let end = s.select(Okay(PL2 {value : true})).await?;
          Result::Ok(((), end))
        }
        Err(UpdateValueErr(acts)) => {
          println!("acts: {:#?}", &acts);
          let end = s.select(Ko(PL3 { value : false })).await?;
          Result::Ok(((), end))
        }
      }
    } else {
      let end = s.select(Ko(PL3 { value : false })).await?;
      Result::Ok(((), end))
    }
  }).await
}

fn isKey<T : cmp::PartialEq,U>(k : T, xs : &Vec<(T,U)>) -> bool {
  for x in xs.iter() {
    if x.0 == k {
      return true
    }
  }
  return false
}

fn lookupKey<T : cmp::PartialEq,U>(k : T, xs : &Vec<(T,U)>) -> Option<&U> {
  for x in xs.iter() {
    if x.0 == k {
      return Some(&x.1);
    }
  }
  return None;
}

#[derive(Debug)]
struct UpdateValueErr<T,U>(Vec<(T,U)>);
impl<T : fmt::Debug,U : fmt::Debug> fmt::Display for UpdateValueErr<T,U> {
  fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
    write!(f, "UpdateValueErr; acts: {:#?}", self.0)
  }
}
impl<T : fmt::Debug,U : fmt::Debug> Error for UpdateValueErr<T,U> {
  fn description(&self) -> &str {
    "UpdateValueErr"
  }
}

fn updateValue<T : cmp::PartialEq + Copy,U : Copy>(k : T, xs : Vec<(T,U)>, f : &dyn Fn(U) -> U)
           -> Result<Vec<(T,U)>,UpdateValueErr<T,U>> {
  if isKey(k, &xs) {
    let xs = xs.iter().map(|x| {
      if x.0 == k {
        (x.0, f(x.1))
      } else {
        (x.0, x.1)
      }
    }).collect();
    Ok(xs)
  } else {
    Err(UpdateValueErr(xs))
  }
}

fn main() {
  let mut roles = Roles::default();
  let xs = vec![(1,10),(2,50)];
  println!("acts0: {:#?}", &xs);
  executor::block_on(async {
    try_join!(c(&mut roles.c), b(&mut roles.b, xs)).unwrap();
  });
}
