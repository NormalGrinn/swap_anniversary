use serde::Serialize;

#[derive(Debug)]
pub struct UserInfo {
    pub discord_id: u64,
    pub username: String,
    pub character_id: u64,
    pub letter: Option<String>,
    pub submission: Option<String>
}

#[derive(Debug, Serialize)]
pub struct Letter {
    pub id: String,
    pub character_image_url: String,
    pub character_name: String,
    pub letter_content: String,
    pub giftee_name: Option<String>,
    pub santa_name: Option<String>,
}

#[derive(Debug)]
pub struct ClaimedLetter {
    pub id: u64,
    pub owner_id: u64,
    pub owner_name: String,
    pub claimee_id: Option<u64>,
    pub claimee_name: Option<String>,
}