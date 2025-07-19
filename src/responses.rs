use std::fmt::Display;

use rocket::{
    http::Status,
    response::{content, status},
};

pub struct ResponseMsg;
pub type Response = status::Custom<content::RawJson<String>>;

impl ResponseMsg {
    pub fn ok() -> Response {
        status::Custom(
            Status::Ok,
            content::RawJson("{\"message\": \"OK\"}".to_string()),
        )
    }

    pub fn err_from<E: Display>(e: E) -> Response {
        status::Custom(
            Status::InternalServerError,
            content::RawJson(format!("{{\"message\": \"{}\"}}", e)),
        )
    }
}
