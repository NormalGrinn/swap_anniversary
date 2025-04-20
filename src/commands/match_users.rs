use std::collections::HashSet;

use crate::{database::{self, claim_letter}, utilities::ensure_host_role, Context, Error};
use poise::CreateReply;
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn match_users(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}
    let users_withouth_giftees = database::get_users_without_giftees().await?;
    let mut the_gifteeless: Vec<u64> = users_withouth_giftees.into_iter().map(|(_name, id)| id).collect();
    
    let mut users_without_santas: Vec<(u64)> = Vec::new();
    let letters = database::get_all_claimed_letters().await?;

    for giftee in &letters {
        if giftee.claimee_id == None { users_without_santas.push((giftee.owner_id));}
    }

    if the_gifteeless.is_empty() || users_without_santas.is_empty() {
        ctx.send(CreateReply::default().content("No one to match").ephemeral(true)).await?;
        return Ok(())
    }

    let gifteeless_set: HashSet<u64> = the_gifteeless.iter().cloned().collect();
    let no_santa_or_giftees: Vec<u64> = users_without_santas
        .iter()
        .filter(|id| gifteeless_set.contains(id))
        .cloned()
        .collect();
    the_gifteeless.retain(|id| !no_santa_or_giftees.contains(id));
    users_without_santas.retain(|id| !no_santa_or_giftees.contains(id));

    if no_santa_or_giftees.len() > 1 {
        for i in 0..no_santa_or_giftees.len() {
            let claimer = no_santa_or_giftees[i];
            let letter_owner = no_santa_or_giftees[(i + 1) % no_santa_or_giftees.len()];
            claim_letter(claimer, letter_owner).await?;
        }
    } else {
        if users_without_santas.is_empty() {
            ctx.send(CreateReply::default().content("Can't match a user with themselves, add someone else to the event!").ephemeral(true)).await?;
            return Ok(())
        }
        claim_letter(no_santa_or_giftees[1], users_without_santas.remove(0)).await?;
        claim_letter(the_gifteeless.remove(1), no_santa_or_giftees[1]).await?;
    }
    for i in 0..the_gifteeless.len() {
        claim_letter(the_gifteeless[i], users_without_santas[i]).await?;
    }

    ctx.send(CreateReply::default().content("Matched users").ephemeral(true)).await?;


    Ok(())
}