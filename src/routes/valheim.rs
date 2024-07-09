use rocket::{response::status, serde::json::Json};
use crate::{types, services};

// using endpoint "the-woodsen", in case future valheim worlds are handled on this server
#[post("/the-woodsen/odin", data = "<odin_body>")]
pub fn handle_odin_request(odin_body: Json<types::valheim::OdinBody>) -> status::NoContent {
    services::valheim::handle(odin_body.into_inner()).unwrap();
    status::NoContent
}

