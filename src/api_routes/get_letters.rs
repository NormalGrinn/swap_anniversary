use std::env;

use warp::Filter;
use serde_json::json;

use crate::database;

pub fn get_letters() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("get_letters")
        .and_then(|| async move {
            let phase = match std::env::var("PHASE") {
                Ok(p) => p,
                Err(_) => return Err(warp::reject()),
            };

            let parsed_phase: u64 = match phase.parse() {
                Ok(p) => p,
                Err(_) => return Err(warp::reject()),
            };

            match database::get_all_letters().await {
                Ok(mut letters) => {
                    for l in &mut letters {
                        if parsed_phase != 4 && l.santa_name.is_some() {
                            l.santa_name = Some("Claimed".to_string());
                        }
                        if l.santa_name.is_none() {
                            l.giftee_name = None;
                        }
                    }
                    let json_reply = json!(letters);
                    Ok(warp::reply::json(&json_reply))
                },
                Err(_) => Err(warp::reject()),
            }
        })
    }