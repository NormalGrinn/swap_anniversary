use std::{env, ops::Deref};

use serenity::all::{CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage, FullEvent, Interaction};

use crate::{database, utilities::ensure_correct_phase, Data, Error};

pub async fn on_component_interaction(
    ctx: &serenity::all::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate { interaction } => {
            match interaction {
                Interaction::Component(component_interaction) => {
                    if component_interaction.data.custom_id == "Join" {
                        let user_id = component_interaction.user.id.get();
                        let response_date = CreateInteractionResponseMessage::new().ephemeral(true);
                        let interaction_response = CreateInteractionResponse::Defer(response_date);
                        component_interaction
                        .create_response(
                            &ctx,
                            CreateInteractionResponse::Defer(
                                CreateInteractionResponseMessage::new().ephemeral(true)
                            )
                        )
                        .await?;
                        let phase = env::var("PHASE")
                        .expect("Missing `PHASE` env var, see README for more information.");
                        let parsed_phase: u64 = phase.parse().expect("Error parsing phase to integer");
                        if parsed_phase != 1 {
                            let message = CreateInteractionResponseFollowup::new()
                            .content("You cannot join in this phase!").ephemeral(true);
                            component_interaction.create_followup(&ctx.http, message).await?;
                            return Ok(())
                        }

                        // component_interaction.create_response(&ctx, interaction_response).await?;
                        
                        let info = database::get_userinfo_by_id(user_id).await;
                        match info {
                            Ok(_) => {
                                let message = CreateInteractionResponseFollowup::new()
                                    .content("You are already in the event!").ephemeral(true);
                                component_interaction.create_followup(&ctx.http, message).await?;
                            },
                            Err(rusqlite::Error::QueryReturnedNoRows) => {
                                database::add_user(&component_interaction.user.name, user_id).await?;
                                let message = CreateInteractionResponseFollowup::new()
                                    .content("Joined the event!").ephemeral(true);
                                component_interaction.create_followup(&ctx.http, message).await?;
                                let join_dm = format!("You have successfully joined the event, you will appear as a random character, **DO NOT** share who this is.");
                                let message = CreateMessage::default().content(join_dm);
                                component_interaction.user.dm(&ctx.http, message).await?;
                                }
                            _ => {
                                let message = CreateInteractionResponseFollowup::new()
                                .content("Unexpected error").ephemeral(true);
                                component_interaction.create_followup(&ctx.http, message).await?;
                            }
                        }
                        Ok(())
                    } else {
                        Ok(())
                    }
                },
                _ => Ok(()),
            }
        },
        _ => Ok(()),
    }

}