use crate::{database::{self, get_giftee, set_submission}, utilities::{self, ensure_dm, ensure_embed_field_lenght, ensure_has_giftee, ensure_joined, wait_for_message_with_cancel}, Context, Error};
use rusqlite::Result;
use serenity::all::{CreateMessage, UserId};

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn submit(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_dm(&ctx).await? {return Ok(())}
    if !ensure_has_giftee(&ctx).await? {return Ok(())}
    match wait_for_message_with_cancel(&ctx, "Send what you want to submit, otherwise press cancel to cancel the action").await? {
        Some(message) => {
            if !ensure_embed_field_lenght(&ctx, &message).await? {return Ok(())}
            match set_submission(ctx.author().id.get(), &message).await {
                Ok(_) => {
                    let giftee = get_giftee(ctx.author().id.get()).await?;
                    let giftee_id = UserId::new(giftee);
                    let user_name: String;
                    match giftee_id.to_user(ctx.http()).await {
                        Ok(u) => {user_name = u.name},
                        Err(_) => {user_name = "giftee".to_string()},
                    }
                    let embed = utilities::embed_builder(&message, 
                        "Your Santa sent you a gift", 
                        &format!("Dear {}", user_name), 
                        "Love, Santa");
                    let giftee_message = CreateMessage::new().embed(embed);
                    match giftee_id.dm(&ctx.http(), giftee_message).await {
                            Ok(_) => {
                                ctx.say("Submission sent to your giftee").await?;
                            },
                            Err(e) => {
                                ctx.say("An error occured sending your message").await?;
                                eprintln!("Error sending message to giftee: {}", e);
                            },
                        }
                },
                Err(e) => {
                    ctx.say("Error with setting your submisison").await?;
                    eprintln!("Error setting submission: {}", e);
                },
            }
        },
        None => {
            return Ok(())
        },
    }
    Ok(())
}