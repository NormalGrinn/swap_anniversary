use crate::{database, utilities::ensure_host_role, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::User;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn match_users(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}
    let users_withouth_giftees = database::get_users_without_giftees().await?;
    let the_gifteeless: Vec<u64> = users_withouth_giftees.into_iter().map(|(_name, id)| id).collect();
    
    let mut users_without_santas: Vec<(String, u64)> = Vec::new();
    let letters = database::get_all_claimed_letters().await?;

    for giftee in &letters {
        if giftee.claimee_id == None { users_without_santas.push((giftee.owner_name.clone(), giftee.owner_id));}
    }

    

    Ok(())
}