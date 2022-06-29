use ::futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
#[allow(unused_imports)]
use ::rumpsteak::{
    channel::Bidirectional, session, Branch, End, Message, Receive, Role, Roles, Select, Send, 
};

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
enum Label {
    Transfer(Transfer),
    Ok(Ok),
    Ko(Ko),
}

struct Transfer(PL1);

struct Ok(PL2);

struct Ko(PL3);

#[session]
type SimpleBankC = Send<B, Transfer, Branch<B, SimpleBankC1>>;

#[session]
enum SimpleBankC1 {
    Ko(Ko, End),
    Ok(Ok, End),
}

#[session]
type SimpleBankB = Receive<C, Transfer, Select<C, SimpleBankB1>>;

#[session]
enum SimpleBankB1 {
    Ko(Ko, End),
    Ok(Ok, End),
}

// -- [Generated above, written below] ----------------------------------------
