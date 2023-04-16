use egui::mutex::RwLock;
use rusqlite::*;
use rusqlite::{params, Connection, Result};
use std::fmt::{Debug, Display};
/// this module manage keys,
///
///
use std::io::*;
use std::sync::Arc;

pub struct Database {
    db: Arc<RwLock<Connection>>,
}

#[derive(PartialEq, Clone)]
pub struct Key {
    pub rowid: i32,
    pub name: String,
    pub sha1: String,
    pub public_key: Option<Vec<u8>>,
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(" - ")?;
        f.write_str(&self.sha1)?;
        Ok(())
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(" - ")?;
        f.write_str(&self.sha1)?;
        Ok(())
    }
}

impl Database {
    pub fn open_database() -> Result<Database> {
        let conn = Connection::open_with_flags(
            "keys.db",
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("Error in opening the database");

        match conn.execute(
            "CREATE TABLE all_keys (
                name TEXT NOT NULL,
                sha1 TEXT NOT NULL,
                public_key BLOB
            )",
            (), // empty list of parameters.
        ) {
            Ok(result) => {}
            Err(e) => println!("table exists : {}", e),
        };

        Ok(Database {
            db: Arc::new(RwLock::new(conn)),
        })
    }

    pub fn insert(&self, k: &Key) -> Result<()> {
        let c = self.db.read();
        c.execute(
            "INSERT INTO all_keys (name, sha1, public_key) VALUES (?1, ?2, ?3)",
            (&k.name, &k.sha1, &k.public_key),
        )?;

        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Key>> {
        let mut v: Vec<Key> = Vec::new();
        let c = self.db.read();
        let mut stmt = c.prepare("SELECT rowid, name, sha1, public_key FROM all_keys")?;
        let keys_iter = stmt.query_map([], |row| {
            Ok(Key {
                rowid: row.get(0)?,
                name: row.get(1)?,
                sha1: row.get(2)?,
                public_key: row.get(3)?,
            })
        })?;

        for k in keys_iter {
            if let Err(e) = k {
                return Err(e);
            } else {
                v.push(k.unwrap());
            }
        }

        Ok(v)
    }
}
