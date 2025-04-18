use crate::{database, utilities::ensure_host_role, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::{CreateAttachment, CreateMessage};
use tokio::{fs::{self, OpenOptions}, io::AsyncWriteExt};
use std::collections::HashSet;

const PATH: &str = "status.txt";

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn status(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}

    let users = database::get_all_users().await?;
    let letters = database::get_all_claimed_letters().await?;

    let mut users_without_letters: Vec<(String, u64)> = Vec::new();
    let mut users_without_santas: Vec<(String, u64)> = Vec::new();
    let mut users_without_submissions : Vec<(String, u64)> = Vec::new();

    for user in &users {
        if user.letter == None { users_without_letters.push((user.username.clone(), user.discord_id)) }
        if user.submission == None {users_without_submissions.push((user.username.clone(), user.discord_id))}
    }
    for giftee in &letters {
        if giftee.claimee_id == None { users_without_santas.push((giftee.owner_name.clone(), giftee.owner_id));}
    }
    let claimed_claimee_ids: HashSet<u64> = letters
        .iter()
        .filter_map(|cl| cl.claimee_id)
        .collect();

    let users_without_giftees: Vec<(u64, String)> = users
        .iter()
        .filter(|user| !claimed_claimee_ids.contains(&user.discord_id))
        .map(|user| (user.discord_id, user.username.clone()))
        .collect();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PATH)
        .await?;
    let mut buffer = String::new();

    buffer.push_str("Users who have not yet written a letter:\n");
    for (id, name) in users_without_letters { buffer.push_str(&format!("{}, {}\n", name, id)) }
    buffer.push_str("\nUsers without Santas:\n");
    for (id, name) in users_without_santas { buffer.push_str(&format!("{}, {}\n", name, id)) }
    buffer.push_str("\nUsers without giftees:\n");
    for (id, name) in users_without_giftees { buffer.push_str(&format!("{}, {}\n", name, id)) }
    buffer.push_str("\nUsers without submissions:\n");
    for (id, name) in users_without_submissions { buffer.push_str(&format!("{}, {}\n", name, id)) }

    fs::write(PATH, buffer.as_bytes()).await?;
    file.flush().await?;

    let attachment = CreateAttachment::path(PATH).await?;
    let builder = CreateMessage::new().add_file(attachment);

    match ctx.author().direct_message(ctx.http(), builder).await {
        Ok(_) => {
            ctx.send(CreateReply::default().content("Successfully sent status").ephemeral(true)).await?;
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error sending status").ephemeral(true)).await?;
        },
    };

    let _ = tokio::fs::remove_file(PATH).await; // Ignore error if the file doesn't exist

    Ok(())
}