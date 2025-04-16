use std::env;
use std::time::Duration;

use poise::serenity_prelude as serenity;
use poise::CreateReply;
use poise::futures_util::{future::select, StreamExt, FutureExt};
use ::serenity::all::CreateActionRow;
use ::serenity::all::CreateButton;
use ::serenity::all::CreateEmbed;
use ::serenity::all::CreateEmbedFooter;
use ::serenity::all::Embed;
use ::serenity::all::EmbedField;
use ::serenity::all::EmbedFooter;
use ::serenity::all::MessageCollector;
use ::serenity::futures::future::Either;
use ::serenity::model::colour;

use crate::database;
use crate::database::get_userinfo_by_id;
use crate::Context;

pub async fn ensure_dm(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let dm_channel = ctx.author().create_dm_channel(&ctx.serenity_context().http).await?;
    let channel_id = dm_channel.id;

    if ctx.channel_id() != channel_id {
        ctx.send(
            CreateReply::default()
                .content("This command is only allowed to be used in DMs")
                .ephemeral(true),
        )
        .await?;
        return Ok(false);
    }

    Ok(true)
}

pub async fn ensure_joined(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let user_id = ctx.author().id.get();
    match get_userinfo_by_id(user_id).await {
        Ok(_) => Ok(true),
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("You have not joined the event yet")
                    .ephemeral(true),
            )
            .await?;
            return Ok(false);
        },
    }
}

pub async fn ensure_has_giftee(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let user_id = ctx.author().id.get();
    match database::check_if_has_claimed(user_id).await {
        Ok(b) => {
            if b {
                Ok(true)
            } else {
                ctx.send(
                    CreateReply::default()
                        .content("You do not have a giftee")
                        .ephemeral(true),
                )
                .await?;
                return Ok(false);
            }
        },
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("An error occured checking if you have a giftee")
                    .ephemeral(true),
            )
            .await?;
            return Ok(false);
        },
    }
}

pub async fn ensure_has_santa(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let user_id = ctx.author().id.get();
    match database::check_if_claimed(user_id).await {
        Ok(b) => {
            if b {
                Ok(true)
            } else {
                ctx.send(
                    CreateReply::default()
                        .content("You do not have a santa")
                        .ephemeral(true),
                )
                .await?;
                return Ok(false);
            }
        },
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("An error occured checking if you have a santa")
                    .ephemeral(true),
            )
            .await?;
            return Ok(false);
        },
    }
}

pub async fn ensure_embed_field_lenght(ctx: &Context<'_>, message: &str) -> Result<bool, serenity::Error> {
    if message.len() > 1000 {
        ctx.send(
            CreateReply::default()
                .content("Your message is over 1000 characters")
                .ephemeral(true),
        )
        .await?;
        return Ok(false);
    }
    Ok(true)
}

pub fn embed_builder(message: &str, title: &str, hello_message: &str, goodbye_message: &str) -> serenity::CreateEmbed {
    let footer = CreateEmbedFooter::new("Swap Anniversary");

    let create_embed = CreateEmbed::new().footer(footer)
        .field(hello_message, message, false)
        .field("", goodbye_message, false)
        .color(colour::Colour::MAGENTA)
        .title(title);

    create_embed
}

pub async fn ensure_host_role(ctx: &Context<'_>, user: &serenity::User) -> Result<bool, serenity::Error> {
    let guild_id_int: u64 = env::var("GUILD_ID")
    .expect("Missing `GUILD_ID` env var, see README for more information.")
    .parse().expect("Error parsing guild id to int");
    let role_id_int: u64 = env::var("HOST_ROLE")
    .expect("Missing `HOST_ROLE` env var, see README for more information.")
    .parse().expect("Error parsing host id to int");

    let guild_id = serenity::GuildId::new(guild_id_int);
    let role_id = serenity::RoleId::new(role_id_int);

    let res = user.has_role(ctx.http(), guild_id, role_id).await?;

    if !res {
        ctx.send(CreateReply::default().content("You do not have the host role").ephemeral(true)).await?;
    }

    Ok(res)
}

pub async fn ensure_correct_phase(ctx: &Context<'_>, allowed_phase: Vec<u64>) -> Result<bool, serenity::Error> {
    let phase = env::var("PHASE")
    .expect("Missing `PHASE` env var, see README for more information.");
    let parsed_phase: u64 = phase.parse().expect("Error parsing phase to integer");
    if !allowed_phase.contains(&parsed_phase) {
        ctx.send(
            CreateReply::default()
                .content("You are trying to run a command that is not allowed in this phase!")
                .ephemeral(true),
        )
        .await?;
        return Ok(false)
    } 
    Ok(true)
}

pub async fn wait_for_message_with_cancel(ctx: &Context<'_>, message_content: &str) -> Result<Option<String>, serenity::Error> {
    let time_out = Duration::from_secs(300);
    let cancel_button = CreateButton::new("Cancel")
        .label("Cancel action")
        .style(serenity::all::ButtonStyle::Danger);
    let buttons: Vec<CreateButton> = vec![cancel_button];
    let action_row = vec![CreateActionRow::Buttons(buttons)];
    let message = CreateReply::default().content(message_content)
    .components(action_row);
    ctx.send(message).await?;

    let user_id = ctx.author().id;
    let channel_id = ctx.channel_id();

    let mut message_future = MessageCollector::new(ctx)
    .author_id(user_id)
    .channel_id(channel_id)
    .timeout(time_out)
    .stream();

    let mut cancel_future = serenity::collector::ComponentInteractionCollector::new(ctx)
    .timeout(time_out)
    .filter(move |interaction| {
        interaction.data.custom_id == "Cancel" && interaction.user.id == user_id
    })
    .stream();

    let message_fut = message_future.next().fuse();
    let cancel_fut = cancel_future.next().fuse();

    let result = select(message_fut, cancel_fut).await;

    match result {
        Either::Left((Some(message), _)) => {
            return Ok(Some(message.content.clone()))
        }
        Either::Right((Some(_cancel), _)) => {
            ctx.say("Command cancelled").await?;
        }
        _ => {
            ctx.say("Command timed out").await?;
        }
    }
    return Ok(None)
}