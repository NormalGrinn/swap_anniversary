use std::env;

use poise::serenity_prelude as serenity;
use dotenvy::dotenv;

mod database;
mod commands;
mod types;
mod components;
mod utilities;
mod api_routes;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("TOKEN")
        .expect("Missing `TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let bot_task = tokio::spawn(async move {
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![ commands::create_button::create_button(),
                                commands::write_letter::write_letter(),
                                commands::read_letter::read_letter(),
                                commands::leave::leave(),
                                commands::remove_user::remove_user(),
                                commands::claim::claim(),
                                commands::write_giftee::write_giftee(),
                                commands::write_santa::write_santa(),
                                commands::set_phase::set_phase(),
                                ],
                event_handler: |ctx, event, framework, data| {
                    Box::pin(components::button_interaction::on_component_interaction(ctx, event, framework, data))
                },
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data {})
                })
            })
            .build();

        let client = serenity::ClientBuilder::new(token, intents)
            .framework(framework)
            .await;
        match client {
            Ok(mut c) => {
                if let Err(e) = c.start().await {
                    eprintln!("Client failed to start: {e:?}");
                }
            }
            Err(e) => {
                eprintln!("Failed to build client: {e:?}");
            }
        }
    });

    let api_task = tokio::spawn(async {
        let letters_routes = api_routes::get_letters::get_letters();
        let routes = letters_routes;
        warp::serve(routes)
            .run(([127, 0, 0, 1], 3030))
            .await
    });
    
    let _ = tokio::join!(bot_task, api_task);
}