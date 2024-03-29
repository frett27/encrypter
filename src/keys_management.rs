use core::fmt;
use egui::mutex::RwLock;
use rusqlite::*;
use rusqlite::{Connection, Result};
use std::error::Error;
use std::fmt::{Debug, Display};

/// this module manage keys,
///
///
use std::sync::Arc;

#[allow(unused_imports)]
use log::{debug, error, info, log_enabled, Level};

pub struct Database {
    db: Arc<RwLock<Connection>>,
}

#[derive(Debug, Clone)]
pub struct KeyManagementError {
    message: String,
}

impl fmt::Display for KeyManagementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyError: {}", &self.message)
    }
}

impl Error for KeyManagementError {
    fn description(&self) -> &str {
        self.message.as_str()
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Key {
    pub rowid: i32,
    pub name: String,
    pub sha1: String,
    pub public_key: Option<Vec<u8>>,
}

pub fn text_representation(k: &Key) -> String {
    String::from("") + &k.name + " (" + &k.sha1 + ")"
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
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS all_keys (
                name TEXT NOT NULL,
                sha1 TEXT NOT NULL PRIMARY KEY,
                public_key BLOB
            )",
            (), // empty list of parameters.
        )?;

        Ok(Database {
            db: Arc::new(RwLock::new(conn)),
        })
    }

    pub fn insert(&self, k: &Key) -> Result<(), Box<dyn Error>> {
        if k.sha1.len() != 40 {
            let s: String = "l'identifiant de la clef doit avoir 40 caractères".into();
            return Err(Box::new(KeyManagementError { message: s }));
        }

        let c = self.db.read();
        c.execute(
            "INSERT or REPLACE INTO all_keys (name, sha1, public_key) VALUES (?1, ?2, ?3)",
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
            v.push(k?);
        }

        Ok(v)
    }
}
