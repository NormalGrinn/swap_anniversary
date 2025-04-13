use rusqlite::{Connection, Result};
use rand::seq::IndexedRandom;

use crate::types;

const PATH: &str = "databases/swapAnniversary.db";

pub async fn add_user(username: &str, user_id: u64) -> Result<()> {
    const ADD_USER: &str = "
    INSERT INTO users (discord_id, username, character_id)
    VALUES (?1, ?2, ?3);
    ";
    const GET_UNCLAIMED_CHARS: &str = "
    SELECT c.character_id
    FROM characters c
    LEFT JOIN users u ON c.character_id = u.character_id
    WHERE u.character_id IS NULL;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut stmt = conn.prepare(GET_UNCLAIMED_CHARS)?;
    let character_ids: Vec<u64> =   stmt.query_map([], |row| row.get(0))?
                                    .collect::<Result<Vec<u64>>>()?;
    let character_id: u64;
    match character_ids.choose(&mut rand::rng()) {
        Some(id) => character_id = *id,
        None => return Ok(()),
    }
    conn.execute(ADD_USER, rusqlite::params![user_id, username, character_id]).map_err(|e| {
        eprintln!("Problem adding user to database: {}", e);
        e
    })?;
    Ok(())
}

pub async fn get_userinfo_by_id(user_id: u64) -> Result<types::UserInfo> {
    const GET_USER: &str = "
    SELECT * FROM users WHERE discord_id = ?1;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let res = conn.query_row(GET_USER, rusqlite::params![user_id], |row|
    Ok(types::UserInfo {
        discord_id: row.get(0)?,
        username: row.get(1)?,
        character_id: row.get(0)?,
        letter: row.get(3)?,
    })
    )?;
    Ok(res)
}

pub async fn set_letter(user_id: u64, letter_content: &str) -> Result<usize> {
    const UPSERT_LETTER: &str = "
    UPDATE users
    SET letter = ?1
    WHERE discord_id = ?2;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let res = conn.execute(UPSERT_LETTER, rusqlite::params![letter_content, user_id])?;
    Ok(res)
}

pub async fn leave(user_id: u64) -> Result<usize> {
    const DELETE_USER: &str = "
    DELETE FROM users
    WHERE discord_id = ?1;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let res = conn.execute(DELETE_USER, rusqlite::params![user_id])?;
    Ok(res)
}