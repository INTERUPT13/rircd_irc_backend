#[macro_use]
extern crate derive_new;

mod endpoint_handler;
mod accept_handler;
mod connect_handler;


use crate::endpoint_handler::IrcEndpointBackend;

#[tokio::main]
async fn main() {
    IrcEndpointBackend::new(vec!["0.0.0.0:7000"], vec![]).unwrap().start().await;
    tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await;
    println!("Hello, world!");
}


#[test]
fn mk_irc_endpoint() {
    assert!(IrcEndpointBackend::new(
        vec!["0.0.0.0:4096"],
        vec![]).is_ok());

    assert!(IrcEndpointBackend::new(
        vec!["0.0.0.0:4096:"],
        vec![]).is_err());

    assert!(IrcEndpointBackend::new(
        vec![],
        vec![]).is_err());
}

use tokio_test::{block_on, assert_ok};

#[test]
fn endpoint_test_bind() {
    // TODO  can this work in the NixOS build ?
    let b = IrcEndpointBackend::new(vec!["0.0.0.0:0"], vec![]);
    //assert_ok!(b);
    assert_ok!(block_on(b.unwrap().start()))
}


