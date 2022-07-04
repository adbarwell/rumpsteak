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
use std::{error::Error, fmt, cmp, sync::{Mutex, Arc, MutexGuard}};

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
// #[derive(Clone, Copy)]
#[derive(Debug)]
enum Label {
  Transfer(Transfer),
  Ok(Okay),
  Ko(Ko),
}

// #[derive(Clone, Copy)]
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

// #[derive(Clone, Copy)]
#[derive(Debug)]
struct PL1 { // u32 = Nat
  accountSrc : Capability<(u32, Capability<u32>)>,
  accountTgt : Capability<(u32, Capability<u32>)>,
  writeSrc : Capability<u32>,
  writeTgt : Capability<u32>,
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
    write!(f, "{:#?}", self)
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

async fn c(role : &mut C,
           r1   : Capability<(u32,Capability<u32>)>,
           r2   : Capability<(u32,Capability<u32>)>,
           w1v  : Capability<u32>,
           rw2v : Capability<u32>)
      -> Result<(), Box<dyn Error>> {
  try_session(role, |s : SimpleBankC<'_, _>|  async {
    let msg : PL1 = PL1 { accountSrc : r2,
                          accountTgt : r1,
                          writeSrc   : rw2v,
                          writeTgt   : w1v,
                          amount     : 10 };
    let s = s.send(Transfer(msg)).await?;
    match s.branch().await? {
      SimpleBankC1::Ko(_, end) => Ok(((), end)),
      SimpleBankC1::Ok(_, end) => Ok(((), end)),
    }


  //   let m : PL1 = PL1 { accountSrc: 1, accountTgt: 2, amount: 10 };
  //   let src = Mutex::new(1);
  //   let x2 = Mutex::new(42);
  //   let c : Capability<u32> = crate::Capability::RCbty(RCbty{ value : x });
  //   let c2 : Capability<u32> = crate::Capability::RCbty(RCbty{ value : x2 });
  //   let y = ceq(c,c2);
  //   println!("y : {:#?}", y);
  //   let s = s.send(Transfer(m)).await?;
  //   match s.branch().await? {
  //     SimpleBankC1::Ok(x, end) => {
  //       Result::Ok(((), end))
  //     }
  //     SimpleBankC1::Ko(y, end) => {
  //       Result::Ok(((), end))
  //     }
  //   }
  }).await
}

// fn b1(x : PL1, acts : Vec<(u32,u32)>) -> Result<Vec<(u32,u32)>, UpdateValueErr<u32,u32>> {
//   let f = |y : u32| -> u32 {y - x.amount};
//   let g = |y : u32| -> u32 {y + x.amount};
//   let acts = updateValue(x.accountSrc, acts, &f)?;
//   updateValue(x.accountTgt, acts, &g)
// }

fn applyFn(x : &mut Capability<u32>, f : &dyn Fn(u32) -> u32) {
  match x {
    Capability::RCbty(_) => todo!(),
    Capability::WCbty(x) => {
      let x1 = &x.value;
      let mut x2 = x1.lock().unwrap();
      *x2 = f(*x2)
    },
    Capability::RWCbty(x) => {
      let x1 = &x.value;
      let mut x2 = x1.lock().unwrap();
      *x2 = f(*x2)
    }
  }
}

async fn b(role : &mut B, acts : Vec<Capability<(u32, Capability<u32>)>>)
      -> Result<(), Box<dyn Error>> {
  try_session(role, |s : SimpleBankB<'_, _>| async {
    let (Transfer(mut x), s) = s.receive().await?;
    println!("c1 : {:#?}", elem(&x.accountSrc, &acts));
    println!("c2 : {:#?}", elem(&x.accountTgt, &acts));
    println!("c3 : {:#?}", getValue(&x.writeSrc) >= Ok(x.amount));
    if elem(&x.accountSrc, &acts)
       && elem(&x.accountTgt, &acts)
       && getValue(&x.writeSrc) >= Ok(x.amount) {
      // let f = |mut y : MutexGuard<u32>| {*y -= &x.amount};
      // let g = |mut y : MutexGuard<u32>| {*y += &x.amount};
      // apply(&mut x.writeSrc, &f)?;

      let y = &mut x.writeSrc;
      let amt = x.amount.clone();
      let f = |z : u32| {z - amt};
      applyFn(y, &f);
      let y = &mut x.writeTgt;
      let amt = x.amount.clone();
      let f = |z : u32| {z + amt};
      applyFn(y, &f);

      // apply(&mut x.writeTgt, &g).unwrap();
      println!("xs : {:#?}", &acts);
      let end = s.select(Okay(PL2 { value : true })).await?;
      Ok(((), end))
    } else {
      todo!()
    }

   //   if isKey(x.accountSrc, &acts)
  //      && isKey(x.accountTgt, &acts)
  //      && lookupKey(x.accountSrc, &acts) >= Some(&x.amount) {
  //     match b1(x, acts) {
  //       Ok(acts) => {
  //         println!("acts: {:#?}", &acts);
  //         let end = s.select(Okay(PL2 {value : true})).await?;
  //         Result::Ok(((), end))
  //       }
  //       Err(UpdateValueErr(acts)) => {
  //         println!("acts: {:#?}", &acts);
  //         let end = s.select(Ko(PL3 { value : false })).await?;
  //         Result::Ok(((), end))
  //       }
  //     }
  //   } else {
  //     let end = s.select(Ko(PL3 { value : false })).await?;
  //     Result::Ok(((), end))
  //   }
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

fn mk_bank_store()
  -> Result<(Arc<Mutex<u32>>,
             Arc<Mutex<u32>>,
             Vec<Capability<(u32, Capability<u32>)>>), CError> {
  let acc1vm : Arc<Mutex<u32>> = Arc::new(Mutex::new(10));
  let acc1v : Capability<u32> =
    Capability::RCbty(RCbty { value : acc1vm.clone() });
  let acc1m : Arc<Mutex<(u32, Capability<u32>)>> =
    Arc::new(Mutex::new((1, acc1v)));
  let acc1 = // bank entry; read-write
    Capability::RWCbty(RWCbty { value : acc1m.clone() });

  let acc2vm : Arc<Mutex<u32>> = Arc::new(Mutex::new(50));
  let acc2v : Capability<u32> =
    Capability::RCbty(RCbty { value : acc2vm.clone() });
  let acc2m : Arc<Mutex<(u32, Capability<u32>)>> =
    Arc::new(Mutex::new((1, acc2v)));
  let acc2 = // bank entry; read-write
    Capability::RWCbty(RWCbty { value : acc2m.clone() });
  let xs = vec![acc1,acc2];
  Ok((acc1vm, acc2vm, xs))
}

fn mk_capabilities_for_c<T>(xs : &Vec<Capability<T>>) -> Result<(Capability<T>, Capability<T>), CError> {
  let r1 = cp_capability(&xs[0])?;
  // let r1 = rmW_capability(r1)?;
  let r2 = cp_capability(&xs[1])?;
  // let r2 = rmW_capability(r2)?;
  Ok((r1, r2))
}

fn main() {
  let mut roles = Roles::default();

  // the bank has a list of account number-value pairs, which it can read from
  // (ignoring new accounts)
  // in order to update an account, it needs a capability to  write to the
  // account balance (which is provided by the user)
  
  // xs : List (Capability (Nat, Capability Nat))
  match mk_bank_store() {
    Ok((m1v, m2v, xs)) => {
      println!("xs : {:#?}", &xs);
      let w1v = Capability::RWCbty(RWCbty { value : m1v });
      let rw2v = Capability::WCbty(WCbty { value : m2v });
      // now, we have a bank store, alongside two capabilities that can 
      // write to the accounts
      // but you also need the ability to prove that you can read/access
      // the accounts you're changing.
      // so we clone both accounts' capabilities & demote them to read only
      let (r1, r2) = mk_capabilities_for_c(&xs).unwrap();
      // now, we have a bank store, two capabilities that will allow us to
      // write to the values of our accounts, and two capabilities that denote
      // the client's access to two accounts
      // next, we need to spawn off our processes
      executor::block_on(async {
        try_join!(c(&mut roles.c, r1, r2, w1v, rw2v),
                  b(&mut roles.b, xs)).unwrap();
      })
    }
    Err(e) => panic!("couldn't create bank store")
  }

  // the client now needs some way of giving permission to modify the thing in 
  // acc1 and acc2; read access only needs to be given on acc1
  // which means that we need the core value of both 1 & 2, which means making
  // the capabilities separately is probably not 


  // // the account number (can be shared)
  // let acc1no : Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
  // let acc1v : Arc<Mutex<(u32,u32)>> =
  //   Arc::new(Mutex::new((1,10)));
  // // acc1 in the database (a pair)
  // let acc1 : Capability<(u32,u32)> =
  //   Capability::RCbty(RCbty { value : acc1v });
  // // acc1 as passed to C (the acc. no.)
  // let acc1c : Capability<u32> =
  //   Capability::RCbty(RCbty { value : acc1no });

  // // the account number (can be shared)
  // let acc2no = Arc::new(Mutex::new(2));
  // let acc2v : Arc<Mutex<(u32,u32)>> =
  //   Arc::new(Mutex::new((2,50)));
  // // acc2 in the database =
  // let acc2 : Capability<(u32,u32)> = Capability::RCbty(RCbty { value : acc2v });
  // // acc2 as passed to C
  // let acc2c : Capability<u32> =
  //   Capability::RCbty(RCbty { value : acc2no });

  // let xs = vec![acc1,acc2];
  // println!("acts0: {:#?}", &xs);
  // executor::block_on(async {
  //   try_join!(c(&mut roles.c, acc1c, acc2c), b(&mut roles.b, xs)).unwrap();
  // });
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
#[derive(Debug)]
struct RCbty<T> {
  value : Arc<Mutex<T>>
}

#[derive(Debug)]
struct WCbty<T> {
  value : Arc<Mutex<T>>
}

#[derive(Debug)]
struct RWCbty<T> {
  value : Arc<Mutex<T>>
}

// A capability is a pointer to/wrapper over T
// Variants are derived by the default permissions
#[derive(Debug)]
enum Capability<T> {
  RCbty(RCbty<T>),
  WCbty(WCbty<T>),
  RWCbty(RWCbty<T>),
  // &c.
}

impl<T : fmt::Display> fmt::Display for Capability<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "Capability")
    }
}

impl<T : PartialEq> PartialEq for Capability<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
          (Self::RCbty(l0), Self::RCbty(r0)) => false,
          _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

fn mk_capability<T>(x : T, r : bool, w : bool, e : bool)
  -> Result<Capability<T>, CError> {
  let v = Arc::new(Mutex::new(x));
  match (r,w,e) {
    (true, true, true) => todo!(),
    (true, true, false) => Ok(Capability::RWCbty(RWCbty { value : v })),
    (true, false, true) => todo!(),
    (true, false, false) => Ok(Capability::RCbty(RCbty { value : v })),
    (false, true, true) => todo!(),
    (false, true, false) => Ok(Capability::WCbty(WCbty { value : v })),
    (false, false, true) => todo!(),
    (false, false, false) => todo!(),
  }
}

fn cp_capability<T>(x : &Capability<T>) -> Result<Capability<T>,CError> {
  if canRead(x) {
    match x {
      Capability::RCbty(y) =>
        Ok(Capability::RCbty(RCbty { value : y.value.clone() })),
      Capability::RWCbty(y) =>
        Ok(Capability::RWCbty(RWCbty { value : y.value.clone() })),
      Capability::WCbty(_) => unimplemented!("impossible case (Capability)")
    }
  } else {
    Err(CError())
  }
}

fn rmW_capability<T>(x : Capability<T>) -> Result<Capability<T>, CError> {
  if canWrite(&x) {
    match x {
    Capability::RCbty(_y) =>
      unimplemented!("impossible case (rmW_capability"),
    Capability::RWCbty(y) =>
      Ok(Capability::RCbty(RCbty { value : y.value.clone() })),
    Capability::WCbty(_y) =>
      todo!("reduces to nothing (rmW_capability"),
    }
  } else {
    Err(CError())
  }
}

// The problem is that we still don't know what operation we can apply to the
// underlying t (that are guarded by the capability)

// Let's assume that we're dealing with an integer (we're sending the reference)
// We therefore want to be able to apply logical operations over the capability
// as well as things like Show (e.g. Show, Eq, Ord)
fn ceq<T : Eq + Copy>(x : Capability<T>, y : Capability<T>) -> Result<bool, CError> {
  if canRead(&x) && canRead(&y) {
    let x = getValue(&x);
    let y = getValue(&y);
    Ok(x == y)
  } else {
    Err(CError())
  }
}

#[derive(Debug)]
struct CError();
impl fmt::Display for CError {
  fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
    write!(f, "CError")
  }
}
impl Error for CError {
  fn description(&self) -> &str {
    "CErr"
  }
}
impl PartialEq for CError {
    fn eq(&self, other: &Self) -> bool {
      false
    }
}
impl PartialOrd for CError {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    // all errors are equal, but some are more equal than others
    Some(cmp::Ordering::Equal)
  }
}


fn canRead<T>(x : &Capability<T>) -> bool {
  match x {
    Capability::RCbty(_y) => true,
    Capability::WCbty(_y) => false,
    Capability::RWCbty(_y) => true
  }
}

fn canWrite<T>(x : &Capability<T>) -> bool {
  match x {
    Capability::RCbty(_y) => false,
    Capability::WCbty(_y) => true,
    Capability::RWCbty(_y) => true
  }
}

fn getValue<T : Copy>(x : &Capability<T>) -> Result<T,CError> {
  match x {
    Capability::RCbty(y) => {
      let z = y.value.lock().unwrap();
      Ok(*z)
    },
    Capability::WCbty(_y) => Err(CError()),
    Capability::RWCbty(y) => {
      let z = y.value.lock().unwrap();
      Ok(*z)
    }
  }
}

fn elem<T>(x : &Capability<T>, ys : &Vec<Capability<T>>) -> bool 
  where T : PartialEq + std::fmt::Debug {
  for y in ys {
    println!("x : {:#?}\ny : {:#?}", &x, &y);
    if x == y { return true }
  }
  return false
}

fn apply<T,S>(x : &mut Capability<T>, f : &dyn Fn(MutexGuard<T>) -> S)
  -> Result<S, CError> {
  todo!()
}

