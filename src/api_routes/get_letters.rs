use warp::Filter;
use serde_json::json;

use crate::database;

pub fn get_letters() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("get_letters")
    .and_then(move || {
        async {
            match database::get_all_letters().await {
                Ok(letters) => {
                    let json_reply = json!(letters);
                    Ok(warp::reply::json(&json_reply))
                },
                Err(_) => Err(warp::reject()),
            }
        }
    })
}