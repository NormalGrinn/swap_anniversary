use std::env;

use warp::Filter;
use serde_json::json;

use crate::database;

pub fn get_letters() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let phase = env::var("PHASE")
        .expect("Missing `PHASE` env var, see README for more information.");
    let parsed_phase: u64 = phase.parse().expect("Error parsing phase to integer");

    warp::path("get_letters")
        .and(warp::any().map(move || parsed_phase))
        .and_then(|parsed_phase| async move {
            match database::get_all_letters().await {
                Ok(mut letters) => {
                    for l in &mut letters {
                        if parsed_phase != 4 && l.santa_name.is_some() {
                            l.santa_name = Some("claimed".to_string());
                        }
                        if l.santa_name.is_none() {
                            l.giftee_name = None
                        }
                    }
                    let json_reply = json!(letters);
                    Ok(warp::reply::json(&json_reply))
                },
                Err(_) => Err(warp::reject()),
            }
        })
}