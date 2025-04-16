use crate::{database, utilities::{ensure_dm, ensure_has_giftee, ensure_joined}, Context, Error};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn read(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_dm(&ctx).await? {return Ok(())}
    if !ensure_has_giftee(&ctx).await? {return Ok(())}

    Ok(())
}