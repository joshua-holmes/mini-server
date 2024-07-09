use rocket::serde;

#[derive(serde::Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct OdinEventType<'r> {
    pub name: &'r str,
    pub status: &'r str,
}
#[derive(serde::Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct OdinBody<'r> {
    pub event_type: OdinEventType<'r>,
    pub event_message: &'r str,
    pub timestamp: &'r str,
}
