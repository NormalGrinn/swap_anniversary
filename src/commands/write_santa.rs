use crate::{database, utilities::{self, ensure_dm, ensure_embed_field_lenght, ensure_has_santa, ensure_joined, wait_for_message_with_cancel}, Context, Error};
use rusqlite::Result;
use serenity::all::{CreateMessage, UserId};

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn write_santa(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_dm(&ctx).await? {return Ok(())}
    if !ensure_has_santa(&ctx).await? {return Ok(())}

    let prompt_message = "
    Press the cancel button to cancel the action, otherwise send a message to send something to your santa (the person giving you something).
    ";
    let user_id = ctx.author().id.get();
    match wait_for_message_with_cancel(&ctx, prompt_message).await? {
        Some(message) => {
            if !ensure_embed_field_lenght(&ctx, &message).await? {return Ok(())}
            match database::get_santa(user_id).await {
                Ok(santa) => {
                    let santa_id = UserId::new(santa);
                    let embed = utilities::embed_builder(&message, 
                        "Your Santa sent you a message", 
                        "Dear Santa", 
                        &format!("Love, {}", ctx.author().name));
                    let santa_message = CreateMessage::new().embed(embed);
                    match santa_id.dm(&ctx.http(), santa_message).await {
                        Ok(_) => {
                            ctx.say("Message sent succesfully to your Santa").await?;
                        },
                        Err(e) => {
                            ctx.say("An error occured sending your message").await?;
                            eprintln!("Error sending message to Santa: {}", e);
                        },
                    }
                },
                Err(e) => {
                    ctx.say("An error occured getting your Santa").await?;
                    eprintln!("Error fetching giftee: {}", e);
                },
            }
        },
        None => {
            return Ok(())
        },
    }

    Ok(())
}