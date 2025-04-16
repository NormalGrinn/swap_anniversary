use crate::{database, utilities::{ensure_has_giftee, ensure_host_role}, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::User;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn status(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}

    Ok(())
}