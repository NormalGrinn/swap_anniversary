use crate::{database, utilities::{ensure_dm, ensure_joined}, Context, Error};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn read_letter(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_dm(&ctx).await? {return Ok(())}

    let user_id = ctx.author().id.get();
    match database::get_userinfo_by_id(user_id).await {
        Ok(user_info) => {
            let letter = user_info.letter;
            match letter {
                Some(l) => {
                    ctx.say(l).await?;
                },
                None => {
                    ctx.say("You have not set your letter yet").await?;
                },
            }
        },
        Err(e) => {
            ctx.say("Error getting letter").await?;
            eprintln!("Error getting letter: {}", e);
        },
    }
    Ok(())
}