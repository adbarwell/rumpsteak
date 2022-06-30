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
use std::{error::Error, fmt};

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
enum Permission {
    Execute,
    Load,
    Store,
    ExecuteC,
    LoadC,
    StoreC,
    Sealable,
    Releasable,
}

trait CReadable {
    fn iAmReadable(&self) -> String {
        "placeholder text (CReadable)".to_string() // ????
    }
}

// This is a very low-level mock-up; presumably we can abstract over this?
// make it a trait that specific 'pointer types' can implement?
#[derive(Clone, Copy)]
#[derive(Debug)]
struct Capability<T> {
    // tag : bool, // whether this is valid or not???
    perms : Permission, // really a 12-element array of booleans
                        // I actually want a vector here -- ask MV how later
                        // Alternatively, we want to make permissions traits?
                        // traits is a silly idea -- they apply to everything
    // uperms : // software defined permissions; can we use this?
                // maybe a vector of boolean functions?
    sealed : bool, // I'm still not sure what practical meaning this has...
                   // sealed = a pointer to a function/closure that we call?
    value : T   // abstraction over offset, address, and length; a pointer?
}

// capabilities need to be encapsulated such that we can only read the value
// if perms includes Load

type CAccessErr = Error;

fn readValue<T>(c : Capability<T>) -> Result<T,CAccessErr> {
    if c.perms == Load {
        Ok(c.value)
    } else {
        Err(CAccessErr)
    }
}

//instead, let's have capabilities be traits that extend ordinary types
// and can be used as things that can control stuff at both the type level and runtime?

#[derive(Clone, Copy)]
#[derive(Debug)]
struct PL1 {
    capability : Capability<u32>, // user has access permissions
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

impl fmt::Display for PL1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PL1( value = {} )", self.value)
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
        let m : PL1 = undefined;
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

async fn b(role : &mut B) -> Result<(), Box<dyn Error>> {
    try_session(role, |s : SimpleBankB<'_, _>| async {
        let (Transfer(x), s) = s.receive().await?;
        let end = s.select(Okay(PL2 { value : true })).await?;
        Result::Ok(((), end))
    }).await
}

fn main() {
    let mut roles = Roles::default();
    executor::block_on(async {
        try_join!(c(&mut roles.c), b(&mut roles.b)).unwrap();
    });
}
