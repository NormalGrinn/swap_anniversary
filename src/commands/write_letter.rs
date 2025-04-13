use crate::{database, utilities::{ensure_dm, ensure_joined, wait_for_message_with_cancel}, Context, Error};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn write_letter(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_dm(&ctx).await? {return Ok(())}

    let prompt_message = "
    Press the cancel button to cancel the action, otherwise send a message to set your letter.
    ";
    let user_id = ctx.author().id.get();
    match wait_for_message_with_cancel(&ctx, prompt_message).await? {
        Some(message) => {
            match database::set_letter(user_id, &message).await {
                Ok(_) => {
                    ctx.say("Letter updated").await?;
                },
                Err(e) => {
                    ctx.say("An error occured").await?;
                    eprintln!("Problem setting letter: {}", e);
                },
            }
        },
        None => {
            return Ok(())
        },
    }

    Ok(())
}