use crate::{database, utilities::ensure_joined, Context, Error};
use poise::CreateReply;
use rust_fuzzy_search::fuzzy_compare;
use serenity::futures;
use ::serenity::futures::Stream;
use rusqlite::Result;

async fn autocomplete_char<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let mut chars = database::get_unclaimed_letter_characters(ctx.author().id.get()).await;
    chars.sort();
    let mut similarity_tuples: Vec<(String, f32)> = chars
        .iter()
        .map(|s| (s.clone(), fuzzy_compare(&partial.to_lowercase(), &s.to_lowercase())))
        .collect();
    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let char_names: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
    futures::stream::iter(char_names)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn claim(
    ctx: Context<'_>,
    #[description = "The name of the letter's writer you want to claim"]
    #[autocomplete = "autocomplete_char"]
    char_name: String
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}

    let author_id = ctx.author().id.get();
    let owner_id = database::get_user_id_by_char_name(char_name).await?;
    if owner_id == author_id {
        ctx.send(CreateReply::default().content("You cannot claim your own letter!").ephemeral(true)).await?;
        return Ok(())
    }

    match database::check_if_claimed(owner_id).await {
        Ok(has_been_claimed) => {
            if has_been_claimed {
                ctx.send(CreateReply::default().content("Letter already has been claimed").ephemeral(true)).await?;
                return Ok(())
            }
        },
        Err(e) => {
            ctx.send(CreateReply::default().content("Error checking if letter has been claimed").ephemeral(true)).await?;
            eprintln!("Error letter checking query: {}", e);
            return Ok(())
        },
    }

    match database::check_if_has_claimed(author_id).await {
        Ok(has_claimed) => {
            if has_claimed {
                ctx.send(CreateReply::default().content("You have already claimed a letter!").ephemeral(true)).await?;
                return Ok(())
            }
        },
        Err(e) => {
            ctx.send(CreateReply::default().content("Error checking if you have already claimed a letter").ephemeral(true)).await?;
            eprintln!("Error letter checking query: {}", e);
            return Ok(())
        },
    }

    match database::claim_letter(author_id, owner_id).await {
        Ok(_) => {
            ctx.send(CreateReply::default().content("Successfully claimed letter").ephemeral(true)).await?;
        },
        Err(e) => {
            ctx.send(CreateReply::default().content("Something went wrong with claiming the letter").ephemeral(true)).await?;
            eprintln!("Error claiming letter: {}", e);
        },
    }
    Ok(())
}