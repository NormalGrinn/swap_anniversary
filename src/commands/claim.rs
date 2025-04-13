use crate::{database, utilities::{ensure_joined}, Context, Error};
use poise::CreateReply;
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn claim(
    ctx: Context<'_>,
    #[description = "The name of the letter's writer you want to claim"]
    letter: String
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    
    Ok(())
}