use crate::{database, utilities::ensure_host_role, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::User;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn remove_user(
    ctx: Context<'_>,
    #[description = "The ID of the user you want to remove"]
    user: User
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}

    let user_id = user.id.get();
    match database::leave(user_id).await {
        Ok(_) => {
            ctx.send(CreateReply::default().content("Succesfully removed the user").ephemeral(true)).await?;
        },
        Err(e) => {
            ctx.send(CreateReply::default().content("Error removing the user").ephemeral(true)).await?;
            eprintln!("Error deleting user: {}", e);
        }
    }
    Ok(())
}