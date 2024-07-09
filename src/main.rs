use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_ec2::Client;

#[macro_use]
extern crate rocket;

mod services;
pub mod types;
mod routes;

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = Client::new(&config);

    let resp = client.describe_instances();
    let things = resp.send().await.unwrap().reservations.unwrap();
    for t in things {
        println!("TING {:?}", t);
    }

    rocket::build().mount("/valheim", routes![routes::valheim::handle_odin_request])
}
