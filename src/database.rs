use rusqlite::{Connection, Result, params};
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
    conn.execute(ADD_USER, params![user_id, username, character_id]).map_err(|e| {
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
    let res = conn.query_row(GET_USER, params![user_id], |row|
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
    const UPDATE_LETTER: &str = "
        UPDATE users
        SET letter = ?1
        WHERE discord_id = ?2;
    ";
    const CHECK_ENTRY: &str = "
        SELECT EXISTS(
            SELECT 1 FROM claimed_letters WHERE owner_id = ?1
        );
    ";
    const CREATE_ENTRY: &str = "
        INSERT INTO claimed_letters (owner_id) VALUES (?1);
    ";

    let mut conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let tx = conn.transaction()?;
    let exists: bool = tx.query_row(CHECK_ENTRY, params![user_id], |row| {
        let value: u64 = row.get(0)?;
        Ok(value != 0)
    })?;
    if !exists {
        tx.execute(CREATE_ENTRY, params![user_id])?;
    }
    let updated = tx.execute(UPDATE_LETTER, params![letter_content, user_id])?;
    tx.commit()?; 

    Ok(updated)
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
    let res = conn.execute(DELETE_USER, params![user_id])?;
    Ok(res)
}

pub async fn get_unclaimed_letter_characters(user_id: u64) -> Vec<String> {
    let mut characters: Vec<String> = Vec::new();
    const GET_UNCLAIMED_LETTER_CHARS: &str = "
    SELECT c.character_name
    FROM users u
    JOIN characters c ON u.character_id = c.character_id
    INNER JOIN claimed_letters claimed_letters ON u.discord_id = claimed_letters.owner_id
    WHERE u.letter IS NOT NULL
        AND claimee_id IS NULL
        AND u.discord_id != (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    }).expect("Error establishing connection");
    let mut char_query = conn.prepare(GET_UNCLAIMED_LETTER_CHARS).expect("Error preparing statement");
    let char_iter = char_query.query_map(params![user_id], |row| {
        let char: String = row.get(0)?;
        Ok(char)
    }).expect("Error querying characters");
    for char in char_iter {
        match char {
            Ok(c) => {
                characters.push(c);
            },
            Err(e) => {
                eprintln!("Error with character: {}", e);
            },
        }
    }
    characters
}

// Should only be used when ensure_joined is used beforehand!
pub async fn get_user_id_by_char_name(char_name: String) -> Result<u64> {
    const ID_BY_CHAR_NAME: &str = "
    SELECT discord_id 
    FROM users
    INNER JOIN characters ON users.character_id = characters.character_id
    WHERE character_name = (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let res = conn.query_row(ID_BY_CHAR_NAME, params![char_name], |row| {
        let id: u64 = row.get(0)?;
        Ok(id)
        })?;
    Ok(res)
}

pub async fn claim_letter(claimer: u64, letter_owner: u64) -> Result<()> {
    const CLAIM_LETTER_QUERY: &str = "
    UPDATE claimed_letters
    SET claimee_id = (?1)
    WHERE owner_id = (?2);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    conn.execute(CLAIM_LETTER_QUERY, params![claimer, letter_owner])?;
    Ok(())
}

pub async fn check_if_claimed(owner_id: u64) -> Result<bool> {
    const CLAIMED_QUERY: &str = "
    SELECT COUNT(claimee_id) FROM claimed_letters WHERE owner_id = ?1
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(CLAIMED_QUERY)?;
    let count: u64 = query.query_row(params![owner_id], |row| row.get(0))?;
    return Ok(count != 0)
}

pub async fn check_if_has_claimed(claimee_id: u64) -> Result<bool> {
    const CLAIMED_QUERY: &str = "
    SELECT COUNT(*) FROM claimed_letters WHERE claimee_id = ?1
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(CLAIMED_QUERY)?;
    let count: u64 = query.query_row(params![claimee_id], |row| row.get(0))?;
    return Ok(count != 0)
}


/*
    pub id: String,
    pub character_image_url: String,
    pub character_name: String,
    pub letter_content: String,
    pub giftee_name: Option<String>,
    pub santa_name: Option<String>,
*/
pub async fn get_all_letters() -> Result<Vec<types::Letter>> {
    const LETTER_QUERY: &str = "
    SELECT
        cl.letter_id,
        c.character_image AS character_image_url,
        c.character_name,
        u.letter AS letter_content,
        u.username AS giftee_name,
        santa.username AS santa_name
    FROM claimed_letters cl
    JOIN users u ON cl.owner_id = u.discord_id
    JOIN characters c ON u.character_id = c.character_id
    LEFT JOIN users santa ON cl.claimee_id = santa.discord_id
    WHERE u.letter IS NOT NULL;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut letter_query = conn.prepare(LETTER_QUERY)?;
    let letter_iter = letter_query.query_map(params!(), |row| {
        let int_id: u64 = row.get(0)?;
        Ok(types::Letter {
            id: int_id.to_string(),
            character_image_url: row.get(1)?,
            character_name: row.get(2)?,
            letter_content: row.get(3)?,
            giftee_name: row.get(4)?,
            santa_name: row.get(5)?,
        })
    })?;
    let mut letters: Vec<types::Letter> = Vec::new();
    for letter in letter_iter {
        match letter {
            Ok(l) => letters.push(l),
            Err(_) => (),
        }
    }
    Ok(letters)
}