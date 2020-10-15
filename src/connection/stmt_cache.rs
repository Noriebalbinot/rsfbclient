//!
//! Rust Firebird Client
//!
//! Statement Cache
//!

use lru_cache::LruCache;
use std::{collections::HashSet, mem};

use crate::{statement::StatementData, Connection, FbError, Transaction};
use rsfbclient_core::FirebirdClient;

/// Cache of prepared statements.
///
/// Must be emptied by calling `close_all` before dropping.
pub struct StmtCache<T> {
    cache: LruCache<String, T>,
    sqls: HashSet<String>,
}

pub struct StmtCacheData<T> {
    pub(crate) sql: String,
    pub(crate) stmt: T,
}

/// General functions
impl<T> StmtCache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
            sqls: HashSet::with_capacity(capacity),
        }
    }

    /// Get a prepared statement from the cache
    fn get(&mut self, sql: &str) -> Option<StmtCacheData<T>> {
        if let Some(stmt) = self.cache.remove(sql) {
            let sql = self.sqls.take(sql).unwrap();

            Some(StmtCacheData { stmt, sql })
        } else {
            None
        }
    }

    /// Adds a prepared statement to the cache, returning the previous one for this sql
    /// or another if the cache is full
    fn insert(&mut self, data: StmtCacheData<T>) -> Option<T> {
        if self.sqls.contains(&data.sql) {
            // Insert the new one and return the old
            self.cache.insert(data.sql, data.stmt)
        } else {
            // Insert the sql
            self.sqls.insert(data.sql.clone());

            // If full, remove the last recently used
            let old = if self.cache.len() == self.cache.capacity() {
                if let Some((sql, stmt)) = self.cache.remove_lru() {
                    // Remove the sql
                    self.sqls.remove(&sql);

                    Some(stmt)
                } else {
                    None
                }
            } else {
                None
            };

            // Insert the new one
            self.cache.insert(data.sql, data.stmt);

            old
        }
    }
}

/// Functions specific for when the data is a `StatementData`
impl<H> StmtCache<StatementData<H>>
where
    H: Send,
{
    /// Get a prepared statement from the cache, or prepare one
    pub fn get_or_prepare<C>(
        tr: &mut Transaction<C>,
        sql: &str,
        named_params: bool,
    ) -> Result<StmtCacheData<StatementData<H>>, FbError>
    where
        C: FirebirdClient<StmtHandle = H>,
    {
        if let Some(data) = tr.conn.stmt_cache.get(sql) {
            Ok(data)
        } else {
            Ok(StmtCacheData {
                sql: sql.to_string(),
                stmt: StatementData::prepare(tr.conn, &mut tr.data, sql, named_params)?,
            })
        }
    }

    /// Adds a prepared statement to the cache, closing the previous one for this sql
    /// or another if the cache is full
    pub fn insert_and_close<C>(
        conn: &mut Connection<C>,
        data: StmtCacheData<StatementData<H>>,
    ) -> Result<(), FbError>
    where
        C: FirebirdClient<StmtHandle = H>,
    {
        conn.stmt_cache.sqls.insert(data.sql.clone());

        // Insert the new one and close the old if exists
        if let Some(mut stmt) = conn.stmt_cache.insert(data) {
            stmt.close(conn)?;
        }

        Ok(())
    }

    /// Closes all statements in the cache.
    /// Needs to be called before dropping the cache.
    pub fn close_all<C>(conn: &mut Connection<C>)
    where
        C: FirebirdClient<StmtHandle = H>,
    {
        let mut stmt_cache = mem::replace(&mut conn.stmt_cache, StmtCache::new(0));

        for (_, stmt) in stmt_cache.cache.iter_mut() {
            stmt.close(conn).ok();
        }
    }
}

#[test]
fn stmt_cache_test() {
    let mut cache = StmtCache::new(2);

    let mk_test_data = |n: usize| StmtCacheData {
        sql: format!("sql {}", n),
        stmt: n,
    };

    let sql1 = mk_test_data(1);
    let sql2 = mk_test_data(2);
    let sql3 = mk_test_data(3);
    let sql4 = mk_test_data(4);
    let sql5 = mk_test_data(5);
    let sql6 = mk_test_data(6);

    assert!(cache.get(&sql1.sql).is_none());

    assert!(cache.insert(sql1).is_none());

    assert!(cache.insert(sql2).is_none());

    let stmt = cache.insert(sql3).expect("sql1 not returned");
    assert_eq!(stmt, 1);

    assert!(cache.get("sql 1").is_none());

    // Marks sql2 as recently used, so 3 must be removed in the next insert
    let sql2 = cache.get("sql 2").expect("Sql 2 not in the cache");
    assert!(cache.insert(sql2).is_none());

    let stmt = cache.insert(sql4).expect("sql3 not returned");
    assert_eq!(stmt, 3);

    let stmt = cache.insert(sql5).expect("sql2 not returned");
    assert_eq!(stmt, 2);

    let stmt = cache.insert(sql6).expect("sql4 not returned");
    assert_eq!(stmt, 4);

    assert_eq!(cache.get("sql 5").expect("sql5 not in the cache").stmt, 5);
    assert_eq!(cache.get("sql 6").expect("sql6 not in the cache").stmt, 6);

    assert!(cache.cache.is_empty());
    assert!(cache.sqls.is_empty());
}
