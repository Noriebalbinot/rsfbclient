//!
//! Rust Firebird Client
//!
//! Example of select with unknown column names
//!
//! You need create a database with this table:
//! create table test (col_a int generated by default as identity, col_b float, col_c varchar(10));
//!
//! You can use the insert example to populate
//! the database ;)
//!

#![allow(unused_variables, unused_mut)]

use rsfbclient::{prelude::*, ConnectionBuilder, FbError, Row};

fn main() -> Result<(), FbError> {
    #[cfg(feature = "linking")]
    let mut conn = ConnectionBuilder::linked()
        .host("localhost")
        .db_name("examples.fdb")
        .user("SYSDBA")
        .pass("masterkey")
        .connect()?;

    #[cfg(feature = "dynamic_loading")]
    let mut conn = ConnectionBuilder::with_client("./fbclient.lib")
        .host("localhost")
        .db_name("examples.fdb")
        .user("SYSDBA")
        .pass("masterkey")
        .connect()?;

    #[cfg(feature = "pure_rust")]
    let mut conn = ConnectionBuilder::pure_rust()
        .host("localhost")
        .db_name("examples.fdb")
        .user("SYSDBA")
        .pass("masterkey")
        .connect()?;

    let rows: Vec<Row> = conn.query("select test.*, 10 as extra from test", ())?;

    for row in rows {
        println!("------------------------------------");

        for col in row.cols {
            println!("{}: {:?}", col.name, col.value);
        }
    }

    Ok(())
}
