#[derive(Debug)]
pub struct UserInfo {
    pub discord_id: u64,
    pub username: String,
    pub character_id: u64,
    pub letter: Option<String>,
}