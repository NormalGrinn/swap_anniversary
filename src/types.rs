use serde::Serialize;

#[derive(Debug)]
pub struct UserInfo {
    pub discord_id: u64,
    pub username: String,
    pub character_id: u64,
    pub letter: Option<String>,
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