use crate::{database, utilities::{ensure_host_role, ensure_no_giftee, ensure_no_santa}, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::User;

/*
Function that should only be enabled if really needed, use /match otherwise
*/
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn set_giftee(
    ctx: Context<'_>,
    #[description = "The ID of the user you want to set as Santa"]
    santa: User,
    #[description = "The ID of the user you want to set as giftee"]
    giftee: User
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}
    if !ensure_no_santa(&ctx, &giftee).await? {return Ok(())}
    if !ensure_no_giftee(&ctx, &santa).await? {return Ok(())}
    if !crate::utilities::ensure_correct_phase(&ctx, vec![2,3]).await? {return Ok(())}

    let santa_check = database::check_if_has_claimed(santa.id.get()).await?;
    let giftee_check = database::check_if_claimed(giftee.id.get()).await?;
    if santa_check || giftee_check {
        ctx.send(CreateReply::default()
        .content("Either the santa already has claimed a letter, or the gifee already has had their letter claimed")
        .ephemeral(true)).await?;
        return Ok(())

    }

    match database::claim_letter(santa.id.get(), giftee.id.get()).await {
        Ok(_) => {
            let message = format!("Successfully set {} as {}'s Santa", santa.name, giftee.name);
            ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
        },
        Err(e) => {
            ctx.send(CreateReply::default().content("Error setting Santa").ephemeral(true)).await?;
        },
    }
    Ok(())
}