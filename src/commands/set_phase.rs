use crate::{utilities::ensure_host_role, Context, Error};
use poise::CreateReply;
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn set_phase(
    ctx: Context<'_>,
    #[description = "The number of the phase you want to set it to"]
    phase: u64,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())};
    if phase > 4 {
        ctx.send(CreateReply::default()
        .content("You are trying to set it to a non-existing phase")
        .ephemeral(true)).await?;
    } else {
        let key = "PHASE";
        std::env::set_var(key, phase.to_string());
        let message = format!("You have successfully set the phase to: {}", phase);
        ctx.send(CreateReply::default()
        .content(message)
        .ephemeral(true)).await?;
    }
    Ok(())
}