use rusqlite::{Connection, Result, params};
use rand::seq::IndexedRandom;

use crate::types::{self, ClaimedLetter, UserInfo};

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
        character_id: row.get(2)?,
        letter: row.get(3)?,
        submission: row.get(4)?,
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

pub async fn leave(user_id: u64) -> Result<()> {
    const DELETE_USER: &str = "
    DELETE FROM users
    WHERE discord_id = ?1;
    ";
    const DELETE_LETTER_ENTRY: &str = "
    DELETE FROM claimed_letters
    WHERE owner_id = ?1;
    ";
    const DELETE_SANTA: & str = "
    UPDATE claimed_letters
    SET claimee_id = NULL
    WHERE claimee_id = ?1;
    ";
    let mut conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let tx = conn.transaction()?;
    tx.execute(DELETE_SANTA, params![user_id])?;
    tx.execute(DELETE_LETTER_ENTRY, params![user_id])?;
    tx.execute(DELETE_USER, params![user_id])?;
    let res = tx.commit()?;
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

// Check if the owner of the letter has had their letter claimed
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

// Check if the user has claimed a letter
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

pub async fn get_giftee(santa_id: u64) -> Result<u64> {
    const GET_GIFTEE: &str = "
    SELECT owner_id 
    FROM claimed_letters
    WHERE claimee_id = (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_GIFTEE)?;
    let giftee: u64 = query.query_row(params![santa_id], |row| row.get(0))?;
    Ok(giftee)
}

pub async fn get_santa(giftee_id: u64) -> Result<u64> {
    const GET_SANTA: &str = "
    SELECT claimee_id
    FROM claimed_letters
    WHERE owner_id = (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_SANTA)?;
    let santa: u64 = query.query_row(params![giftee_id], |row| row.get(0))?;
    Ok(santa)
}

pub async fn get_character_name_and_image(user_id: u64) -> Result<(String, String)> {
    const GET_CHAR_NAME: &str = "
    SELECT characters.character_name, character_image
    FROM characters
    INNER JOIN users USING (character_id)
    WHERE users.discord_id = (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_CHAR_NAME)?;
    let (char_name, char_url): (String, String) = query.query_row(params![user_id], 
        |row| 
        Ok((
            row.get(0)?,
            row.get(1)?)
        )
        )?;
    Ok((char_name, char_url))
}

pub async fn get_giftee_letter(santa_id: u64) -> Result<Option<String>> {
    const GET_LETTER: &str = "
    SELECT u.letter
    FROM claimed_letters cl
    JOIN users u ON cl.owner_id = u.discord_id
    WHERE cl.claimee_id = ?;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_LETTER)?;
    let letter = query.query_row(params![santa_id], 
        |row| 
        Ok(row.get(0)?
        ))?;
    Ok(letter)
}

pub async fn get_giftee_name(santa_id: u64) -> Result<String> {
    const GET_NAME: &str = "
    SELECT u.username
    FROM claimed_letters cl
    JOIN users u ON cl.owner_id = u.discord_id
    WHERE cl.claimee_id = ?;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_NAME)?;
    let name = query.query_row(params![santa_id], 
        |row| 
        Ok(row.get(0)?
        ))?;
    Ok(name)
}

pub async fn set_submission(santa_id: u64, submission_content: &str) -> Result<()> {
    const UPDATE_SUBMISSION: &str = "
    UPDATE users
    SET submitted_gift = ?1
    WHERE discord_id = ?2
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(UPDATE_SUBMISSION)?;
    query.execute(params![submission_content, santa_id])?;
    Ok(())
}

pub async fn get_all_users() -> Result<Vec<UserInfo>> {
    const GET_USERS: &str = "
    SELECT * FROM users;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_USERS)?;
    let user_iter = query.query_map((), |row|
    Ok(UserInfo {
        discord_id: row.get(0)?,
        username: row.get(1)?,
        character_id: row.get(2)?,
        letter: row.get(3)?,
        submission: row.get(4)?,
    })
    )?;
    let mut users: Vec<UserInfo> = Vec::new();
    for u in user_iter {
        match u {
            Ok(user) => {users.push(user)},
            Err(_) => {continue;},
        }
    }
    Ok(users)
}

pub async fn get_all_claimed_letters() -> Result<Vec<ClaimedLetter>> {
    const GET_CLAIMED_LETTERS: &str = "
    SELECT
        cl.letter_id,
        owner.discord_id AS owner_id,
        owner.username AS owner_name,
        claimee.discord_id AS claimee_id,
        claimee.username AS claimee_name
    FROM claimed_letters cl
    JOIN users owner ON cl.owner_id = owner.discord_id
    LEFT JOIN users claimee ON cl.claimee_id = claimee.discord_id;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_CLAIMED_LETTERS)?;
    let letter_iter = query.query_map((), |row|
    Ok(ClaimedLetter {
        id: row.get(0)?,
        owner_id: row.get(1)?,
        owner_name: row.get(2)?,
        claimee_id: row.get(3)?,
        claimee_name: row.get(4)?,
    })
    )?;
    let mut letters: Vec<ClaimedLetter> = Vec::new();
    for l in letter_iter {
        match l {
            Ok(letter) => {letters.push(letter)},
            Err(_) => {continue},
        }
    }
    Ok(letters)
}

pub async fn get_users_without_giftees() -> Result<Vec<(String, u64)>> {
    const GET_USERS_WITHOUT_GIFTEES:  &str = "
    SELECT username, discord_id
    FROM users
    WHERE discord_id NOT IN (
        SELECT claimee_id
        FROM claimed_letters
        WHERE claimee_id IS NOT NULL
    );
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_USERS_WITHOUT_GIFTEES)?;
    let mut user_iter = query.query_map((), |row| {
        let name: String = row.get(0)?;
        let id: u64 = row.get(1)?;
        Ok((name, id))
    })?;
    let mut users: Vec<(String, u64)> = Vec::new();
    for user in user_iter {
        match user {
            Ok(u) => { users.push(u);},
            Err(_) => { continue },
        }
    }
    Ok(users)
}