#[macro_use]
extern crate rocket;

mod responses;
mod services;

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    rocket::build()
        .mount("/ip-logger/log", routes![services::ip_logger::log_ip])
        .mount("/ip-logger", routes![services::ip_logger::serve_html])
}
