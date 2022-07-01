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
use std::{error::Error, fmt, cmp, sync::Mutex};

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


// We need some sort of representation of capabilities in Rust.
// These will presumably be part of a library and directly call the actual
// capabilities. This means, we can take a little liberty in re representation.
// We presumably want to generate a capability over a value.
// We want Traits to indicate which operations we can apply to a capability
// and thus the underlying value by proxy.
// This means, we should really have some sort of pointer within the capability
// to the value itself, rather than just including the value in the capability
// directly.

/* This is too complicated and Rust doesn't support quite the sort of things
   I'm thinking of/hoping for.
#[derive(Debug)]
enum Perms {
  Read,
  Write,
  Execute,
  // ...
}

trait Op<T> {
  fn op(t: T) -> bool;
}

// T is the base value type.
// We need some way of representing the idea that we are pointing at the value
// this can be done either by mutexes or by reference counters (I think)
// I understand mutexes from C, so let's assume these for now.
struct Capability<T> {
  perms : Vec<Perms>,
  traits : Vec<Fn(T) -> bool>,
  value : Mutex<T>
}

fn op (x : Capability<T>) -> a {
  for op in x.traits {
    op.op(x.value.get_mut())
  }
}

trait READ{};

struct RCapability{
  ...
}

impl READ for RCapability{}

p -> q : tf(capability<READ, int> )

q() {
  reeive p {
    label(x) ->

  }
}


fn receive(C<int>) -> ReadAccess<int>
where 
  C: Read,
{

}
*/


// The core part of a capability -- this one only has read enabled
struct RCbty<T> {
  value : Mutex<T>
}

// A capability is a pointer to/wrapper over T
// Variants are derived by the default permissions
enum Capability<T> {
  RCbty(RCbty<T>),
  WCbty,
  RWCbty,
  // &c.
}

// The problem is that we still don't know what operation we can apply to the
// underlying t (that are guarded by the capability)

// Let's assume that we're dealing with an integer (we're sending the reference)
// We therefore want to be able to apply logical operations over the capability
// as well as things like Show (e.g. Show, Eq, Ord)
fn ceq(x : Capability<i32>, y : Capability<i32>) -> Result<bool, CError> {
  if canRead(&x) && canRead(&y) {
    let x = getValue(x);
    let y = getValue(y);
    Ok(x == y)
  } else {
    todo!()
  }
}

#[derive(Debug)]
struct CError();
impl<T : fmt::Debug,U : fmt::Debug> fmt::Display for CError {
  fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
    write!(f, "CError")
  }
}
impl<T : fmt::Debug,U : fmt::Debug> Error for CError {
  fn description(&self) -> &str {
    "CErr"
  }
}


fn canRead<T>(x : &Capability<T>) -> bool {
  match x {
    Capability::RCbty(_y) => true,
    Capability::WCbty => false,
    Capability::RWCbty => true
  }
}

fn getValue<T>(x : Capability<T>) -> Result<T,CError> {
  match x {
    Capability::RCbty(y) => {
      let z = sync::Mutex::get_mut(y);
      todo!()
      // match y.value.lock() {
      //   Ok(z) => {
      //     let w = z.unwrap();
      //     todo!()
      //   },
      //   Err(_) => todo!(),
      // }
    },
    Capability::WCbty => todo!(),
    Capability::RWCbty => todo!()
  }
}

