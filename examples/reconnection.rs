//!
//! Rust Firebird Client
//!
//! Example of simple reconnection
//!
//! You need create a database named test.fdb:
//!

#![allow(unused_variables, unused_mut)]

use rsfbclient::{prelude::*, ConnectionBuilder};
use std::time::Duration;

fn main() {
    let builder = {
        #[cfg(feature = "linking")]
        let mut builder = ConnectionBuilder::linked();

        #[cfg(feature = "dynamic_loading")]
        let mut builder = ConnectionBuilder::with_client("./fbclient.lib");

        #[cfg(feature = "pure_rust")]
        let mut builder = ConnectionBuilder::pure_rust();

        builder
            .host("localhost")
            .db_name("test.fdb")
            .user("SYSDBA")
            .pass("masterkey");

        builder
    };

    let mut conn = builder.connect().unwrap();

    loop {
        match conn.query_first("SELECT rand() FROM RDB$DATABASE", ()) {
            Ok(Some((resp,))) => {
                let resp: f64 = resp;

                println!("Resp: {}", resp);
            }

            Err(e) => {
                eprintln!("Error: {}", e);

                match builder.connect() {
                    Ok(new_conn) => conn = new_conn,
                    Err(e) => eprintln!("Error on reconnect: {}", e),
                }
            }

            _ => panic!("Select returned nothing"),
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
