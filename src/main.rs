use rocket::{response::status, serde::json::Json};

#[macro_use]
extern crate rocket;

mod services;
pub mod types;

// using this endpoint, which is the name of the world, in case future valheim worlds are handled here
#[post("/the-woodsen", data = "<odin_body>")]
fn valheim_route(odin_body: Json<types::valheim::OdinBody>) -> status::NoContent {
    services::valheim::handle(odin_body.into_inner()).unwrap();
    status::NoContent
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/valheim", routes![valheim_route])
}
