use std::net::IpAddr;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::responses::{Response, ResponseMsg};
use chrono::offset::Utc;
use chrono::DateTime;
use rocket::request::{FromRequest, Outcome};
use rocket::response::{content, status};
use rocket::Request;

#[derive(Clone)]
pub struct ClientIp(Option<IpAddr>);

#[derive(Clone)]
pub struct IpCsvPath(PathBuf);

const MAX_LINES_IN_CSV: usize = 1000;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientIp {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.client_ip() {
            Some(ip) => Outcome::Success(ClientIp(Some(ip))),
            None => Outcome::Success(ClientIp(None)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IpCsvPath {
    type Error = ();

    async fn from_request(_: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push("output/ips.csv");
        Outcome::Success(IpCsvPath(path))
    }
}

#[get("/")]
pub fn serve_html() -> status::Accepted<content::RawHtml<String>> {
    status::Accepted(content::RawHtml(
        r#"
<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>IP Logger</title>
    <script>
        fetch("/ip-logger/log")
            .then(r => {
                console.log("status", r.status);
                return r.text();
            })
            .then(d => console.log("response", d))
            .catch(e => console.error("BIG OOPS", e));
    </script>
</head>

<body>
    I have logged your IP. Thank you for participating in this cyber security experiment!
</body>

</html>
    "#
        .to_string(),
    ))
}

#[get("/")]
pub fn log_ip(ip_addr: ClientIp, csv_path: IpCsvPath) -> Result<Response, Response> {
    let timestamp = SystemTime::now();
    let datetime: DateTime<Utc> = timestamp.into();
    let format = "%Y-%m-%dT%T";

    let ip = match ip_addr.0 {
        Some(ip) => ip.to_canonical().to_string(),
        None => String::from("null"),
    };

    let path = csv_path.0;
    let fields = "ip address,timestamp\n";
    let mut line = format!("{},{}", ip, datetime.format(format)).into_bytes();

    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(ResponseMsg::err_from)?;
    }
    if !std::fs::exists(&path).map_err(ResponseMsg::err_from)? {
        std::fs::write(&path, fields).map_err(ResponseMsg::err_from)?;
    }

    // slim down csv so we don't just append and append until storage is used up
    let contents = String::from_utf8(std::fs::read(&path).map_err(ResponseMsg::err_from)?)
        .map_err(ResponseMsg::err_from)?;
    let mut lines: Vec<&str> = contents.lines().collect();
    while lines.len() > MAX_LINES_IN_CSV - 1 {
        lines.remove(1);
    }
    let mut contents: Vec<u8> = lines.join("\n").into_bytes();

    // add new line (10) because `.lines()` removes it
    contents.push(10);

    contents.append(&mut line);

    std::fs::write(&path, contents).map_err(ResponseMsg::err_from)?;

    Ok(ResponseMsg::ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_valid_ip() {
        let ip = ClientIp(Some("1.2.3.4".parse::<IpAddr>().unwrap()));
        let path = IpCsvPath(PathBuf::from("test_logs_valid_ip.csv"));
        std::fs::remove_file(&path.0).unwrap_or(());

        let result = log_ip(ip, path.clone());

        assert!(result.is_ok(), "Should succeed logging valid IP");

        let contents = std::fs::read_to_string(&path.0).expect("File should exist");
        assert!(
            contents.contains("ip address,timestamp\n"),
            "Header should be written"
        );
        assert!(contents.contains("1.2.3.4"), "IP should be written");

        std::fs::remove_file(&path.0).unwrap();
    }

    #[test]
    fn test_logs_null_for_missing_ip() {
        let ip = ClientIp(None);
        let path = IpCsvPath(PathBuf::from("test_logs_null_for_missing_ip.csv"));
        std::fs::remove_file(&path.0).unwrap_or(());

        let result = log_ip(ip, path.clone());

        assert!(result.is_ok(), "Should succeed logging null IP");

        let contents = std::fs::read_to_string(&path.0).expect("File should exist");
        assert!(
            contents.contains("ip address,timestamp\n"),
            "Header should be written"
        );
        assert!(contents.contains("null"), "Null IP should be logged");

        std::fs::remove_file(&path.0).unwrap();
    }

    #[test]
    fn test_subsequent_entries_are_appended() {
        let ip1 = ClientIp(Some("1.1.1.1".parse().unwrap()));
        let ip2 = ClientIp(Some("2.2.2.2".parse().unwrap()));
        let path = IpCsvPath(PathBuf::from("test_subsequent_entries_are_appended.csv"));
        std::fs::remove_file(&path.0).unwrap_or(());

        log_ip(ip1, path.clone()).unwrap();
        log_ip(ip2, path.clone()).unwrap();

        let contents = std::fs::read_to_string(&path.0).expect("File should exist");

        let header = "ip address,timestamp\n";
        let index = contents.find(header).expect("Header should be written");
        assert!(contents.contains("1.1.1.1"), "First IP should be in file");
        assert!(contents.contains("2.2.2.2"), "Second IP should be in file");

        let contents = contents[0..index].to_string() + &contents[index + header.len()..];
        assert!(!contents.contains(header), "Only writes header once");

        std::fs::remove_file(&path.0).unwrap();
    }

    #[test]
    fn test_file_line_length_is_capped() {
        let ip = ClientIp(Some("1.2.3.4".parse::<IpAddr>().unwrap()));
        let path = IpCsvPath(PathBuf::from("test_file_line_length_is_capped.csv"));
        std::fs::remove_file(&path.0).unwrap_or(());

        for _ in 0..(MAX_LINES_IN_CSV + 10) {
            log_ip(ip.clone(), path.clone()).unwrap();
        }

        let contents = String::from_utf8(std::fs::read(&path.0).unwrap()).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert!(
            lines.len() == MAX_LINES_IN_CSV,
            "File line length was not capped"
        );

        std::fs::remove_file(&path.0).unwrap();
    }
}
