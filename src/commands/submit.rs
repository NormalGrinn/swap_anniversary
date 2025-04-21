use crate::{
    database::{get_giftee, set_submission},
    utilities::{
        self, ensure_dm, ensure_embed_field_lenght, ensure_has_giftee, ensure_joined, reject_if_already_running, wait_for_message_with_cancel
    },
    Context, Error,
};
use rusqlite::Result;
use serenity::all::{CreateMessage, UserId};

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn submit(ctx: Context<'_>) -> Result<(), Error> {
    reject_if_already_running(&ctx, || async {
        if !ensure_joined(&ctx).await? { return Ok(()); }
        if !ensure_dm(&ctx).await? { return Ok(()); }
        if !ensure_has_giftee(&ctx).await? { return Ok(()); }

        match wait_for_message_with_cancel(
            &ctx,
            "Send what you want to submit, otherwise press cancel to cancel the action",
        )
        .await?
        {
            Some(message) => {
                if !ensure_embed_field_lenght(&ctx, &message).await? {
                    return Ok(());
                }

                match set_submission(ctx.author().id.get(), &message).await {
                    Ok(_) => {
                        let giftee = get_giftee(ctx.author().id.get()).await?;
                        let giftee_id = UserId::new(giftee);
                        let user_name = match giftee_id.to_user(ctx.http()).await {
                            Ok(u) => u.name,
                            Err(_) => "giftee".to_string(),
                        };

                        let embed = utilities::embed_builder(
                            &message,
                            "Your Santa sent you a gift",
                            &format!("Dear {}", user_name),
                            "Love, Santa",
                        );

                        let giftee_message = CreateMessage::new().embed(embed);

                        let _ = match giftee_id.dm(&ctx.http(), giftee_message).await {
                            Ok(_) => ctx.say("Submission sent to your giftee").await?,
                            Err(e) => {
                                eprintln!("Error sending message to giftee: {}", e);
                                ctx.say("An error occurred sending your message").await?
                            }
                        };
                    }
                    Err(e) => {
                        eprintln!("Error setting submission: {}", e);
                        ctx.say("Error with setting your submission").await?;
                    }
                }
            }
            None => {
                ctx.say("Cancelled.").await?;
            }
        }

        Ok(())
    }).await

}
