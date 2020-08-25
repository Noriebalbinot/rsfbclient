//!
//! Rust Firebird Client
//!
//! Example of select using the embedded firebird server
//!

#![allow(unused_variables, unused_mut, unreachable_code, unused_imports)]

use rsfbclient::{prelude::*, ConnectionBuilder, FbError};

fn main() -> Result<(), FbError> {
    #[cfg(not(feature = "pure_rust"))] // No support for embedded with pure rust driver
    {
        #[cfg(feature = "linking")]
        let mut conn = ConnectionBuilder::linked()
            .embedded()
            .db_name("/tmp/examples.fdb")
            .user("SYSDBA")
            .connect()?;

        #[cfg(feature = "dynamic_loading")]
        let mut conn = ConnectionBuilder::with_client("./fbclient.lib")
            .embedded()
            .db_name("/tmp/examples.fdb")
            .user("SYSDBA")
            .connect()?;

        let rows: Vec<(String, String)> = conn.query(
            "select mon$attachment_name, mon$user from mon$attachments",
            (),
        )?;

        for row in rows {
            println!("Attachment {}, user {}", row.0, row.1);
        }
    }
    Ok(())
}
