use ::futures::{
  channel::mpsc::{UnboundedReceiver, UnboundedSender},
  executor, try_join
};
#[allow(unused_imports)]
use ::rumpsteak::{
  channel::Bidirectional, session, Branch, End, Message, Receive, Role, Roles, Select, Send, 
  try_session
};

use std::{error::Error, fmt, cmp, sync::{Mutex, Arc}};

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
  accountSrc : Capability<(u32, Arc<Mutex<u32>>)>,
  accountTgt : Capability<(u32, Arc<Mutex<u32>>)>,
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

// impl fmt::Display for Label {
//   fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
//     write!(f, "label: {}", self.to_string())
//   }
// }

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
           r1   : Capability<(u32,Arc<Mutex<u32>>)>,
           r2   : Capability<(u32,Arc<Mutex<u32>>)>,
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
  }).await
}

// side effecting
fn applyFn(x : &mut Capability<u32>, f : &dyn Fn(u32) -> u32) -> () {
  match x {
    Capability::RCbty(_) => unimplemented!("impossible case (applyFn"),
    Capability::WCbty(x) => {
      let x1 = &x.value;
      let mut x2 = x1.lock().unwrap();
      *x2 = f(*x2);
      ()
    },
    Capability::RWCbty(x) => {
      let x1 = &x.value;
      let mut x2 = x1.lock().unwrap();
      *x2 = f(*x2);
      ()
    }
  }
}

async fn b(role : &mut B, acts : Vec<Capability<(u32, Arc<Mutex<u32>>)>>)
      -> Result<(), Box<dyn Error>> {
  try_session(role, |s : SimpleBankB<'_, _>| async {
    let (Transfer(mut x), s) = s.receive().await?;
    // println!("c1 : {:#?}", elem(&x.accountSrc, &acts));
    // println!("c2 : {:#?}", elem(&x.accountTgt, &acts));
    // println!("c3 : {:#?}", getValue(&x.writeSrc) >= Ok(x.amount));
    let t0 : Result<bool,CError> = {
      let x0 = getValue(&x.accountSrc)?.1;
      let x0 = x0.lock().unwrap();
      Ok(elem(&x.accountSrc, &acts)
         && elem(&x.accountTgt, &acts)
         && *x0 >= x.amount)
    };
    // println!("amt0: {:#?}", amt0);
    if t0.unwrap_or(false) {
        let y = &mut x.writeSrc;
        let amt = x.amount.clone();
        let f = &|z : u32| {z.clone().checked_sub(amt).unwrap_or(0)};
        let _ = applyFn(y, f);
        let y = &mut x.writeTgt;
        let amt = x.amount.clone();
        let f = |z : u32| {z + amt};
        let _ = applyFn(y, &f);

        println!("xs : {:#?}", &acts);
        let end = s.select(Okay(PL2 { value : true })).await?;
        Ok(((), end))
      } else {
        let end = s.select(Ko(PL3 { value : false })).await?;
        Ok(((), end))
      }
  }).await
}

fn mk_bank_store()
  -> Result<(Arc<Mutex<u32>>,
             Arc<Mutex<u32>>,
             Vec<Capability<(u32, Arc<Mutex<u32>>)>>), CError> {
  let acc1vm : Arc<Mutex<u32>> = Arc::new(Mutex::new(10));
  // let acc1v : Capability<u32> =
  //   Capability::RCbty(RCbty { value : acc1vm.clone() });
  let acc1m : Arc<Mutex<(u32, Arc<Mutex<u32>>)>> =
    Arc::new(Mutex::new((1, acc1vm.clone())));
  let acc1 = // bank entry; read-write
    Capability::RWCbty(RWCbty { value : acc1m.clone() });

  let acc2vm : Arc<Mutex<u32>> = Arc::new(Mutex::new(50));
  // let acc2v : Capability<u32> =
  //   Capability::RCbty(RCbty { value : acc2vm.clone() });
  let acc2m : Arc<Mutex<(u32, Arc<Mutex<u32>>)>> =
    Arc::new(Mutex::new((2, acc2vm.clone())));
  let acc2 = // bank entry; read-write
    Capability::RWCbty(RWCbty { value : acc2m.clone() });

  let acc3vm : Arc<Mutex<u32>> =
    Arc::new(Mutex::new(100));
  let acc3 = // bank entry; read-write
    mk_capability((3, acc3vm), true, true, false).unwrap();

  let xs = vec![acc1,acc2,acc3];
  Ok((acc1vm, acc2vm, xs))
}

fn mk_capabilities_for_c<T>(xs : &Vec<Capability<T>>) -> Result<(Capability<T>, Capability<T>), CError> {
  let r1 = cp_capability(&xs[0])?;
  let r1 = rmW_capability(r1)?;
  let r2 = cp_capability(&xs[1])?;
  let r2 = rmW_capability(r2)?;
  Ok((r1, r2))
}

fn main() {
  let mut roles = Roles::default();

  // xs : List (Capability (Nat, Capability Nat))
  match mk_bank_store() {
    Ok((m1v, m2v, xs)) => {
      println!("xs : {:#?}", &xs);
      let w1v = Capability::RWCbty(RWCbty { value : m1v });
      let rw2v = Capability::WCbty(WCbty { value : m2v });
      let (r1, r2) = mk_capabilities_for_c(&xs).unwrap();
      executor::block_on(async {
        try_join!(c(&mut roles.c, r1, r2, w1v, rw2v),
                  b(&mut roles.b, xs)).unwrap();
      })
    }
    Err(e) => panic!("couldn't create bank store")
  }

}

// [Draft Capability Implementation] ------------------------------------------
// With the real capability interface, we won't necessarily be using mutexes
// but for now, they approximate what we want
// Functions/actions applied to the region of memory pointed at by a capability
// must go via the capability interface -- if you have the core mutex, you can
// bypass it, but with the real implementation, this shouldn't be a problem.
// Capabilities here are very focussed on access permissions to illustrate
// the idea -- this could be expanded out according to the other things
// capabilities afford us (such as...?)

// The core part of a capability -- this one only has read enabled
#[derive(Debug)]
#[derive(Clone)]
struct RCbty<T> {
  value : Arc<Mutex<T>>
}

#[derive(Debug)]
#[derive(Clone)]
struct WCbty<T> { // write only
  value : Arc<Mutex<T>>
}

#[derive(Debug)]
#[derive(Clone)]
struct RWCbty<T> { // read-write
  value : Arc<Mutex<T>>
}

// The main interface type for capabilities; the underlying representations
// are just a means to discriminate between permissions
// A capability is a pointer to/wrapper over T
// Variants are derived by the default permissions
#[derive(Debug)]
#[derive(Clone)]
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

fn mk_capability<T>(x : T, r : bool, w : bool, e : bool)
  -> Result<Capability<T>, CError> {
  let v = Arc::new(Mutex::new(x));
  match (r,w,e) {
    (true, true, false) => Ok(Capability::RWCbty(RWCbty { value : v })),
    (true, false, false) => Ok(Capability::RCbty(RCbty { value : v })),
    (false, true, false) => Ok(Capability::WCbty(WCbty { value : v })),
    // other cases...
    (true, true, true) => todo!(),
    (true, false, true) => todo!(),
    (false, true, true) => todo!(),
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
      Err(CError())
    }
  } else {
    Err(CError())
  }
}

// two capabilities are equivalent if their base values are equal
// the capabilities may have different permissions
fn equiv_capability<T>(x : &Capability<T>, y : &Capability<T>)
                    -> Result<bool,CError> {
  // if you can't read the capabilities, you can't tell if they're equivalent
  if canRead(x) && canRead(y) { 
    let xm = getPtr(x)?;
    let ym = getPtr(y)?;
    Ok(Arc::ptr_eq(xm, ym))
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

fn getValue<T : Clone>(x : &Capability<T>) -> Result<T,CError> {
  match x {
    Capability::RCbty(y) => {
      let z = y.value.lock().unwrap();
      Ok((&*z).clone())
    },
    Capability::WCbty(_y) => Err(CError()),
    Capability::RWCbty(y) => {
      let z = y.value.lock().unwrap();
      Ok((&*z).clone())
    }
  }
}

fn getPtr<T> (x : &Capability<T>) -> Result<&Arc<Mutex<T>>,CError> {
  match x {
    Capability::RCbty(y) => {
      Ok(&y.value)
    },
    Capability::WCbty(_y) => Err(CError()),
    Capability::RWCbty(y) => {
      Ok(&y.value)
    }
  }
}

fn elem<T>(x : &Capability<T>, ys : &Vec<Capability<T>>) -> bool 
  where T : std::fmt::Debug {
  for y in ys {
    let z = equiv_capability(x, y).unwrap();
    if z { return true }
  }
  return false
}
