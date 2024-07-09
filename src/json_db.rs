use std::{fs, io};

use rocket::serde::{json, Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    CannotAccessFilesystem(io::Error),
    CannotWriteToFilesystem(io::Error),
    FailedToParseJson(json::serde_json::Error),
    FailedToSerializeIntoJson(json::serde_json::Error),
}
pub struct Client<T> {
    url: String,
    data: Option<String>,
    json_data: Option<T>,
}
impl<'a, T> Client<T>
where
    T: Deserialize<'a> + Serialize + Clone,
{
    pub fn init(url: &str) -> Result<Self, Error> {
        let exists = fs::try_exists(url).map_err(|e| Error::CannotAccessFilesystem(e))?;

        if !exists {
            fs::write(url, "").map_err(|e| Error::CannotWriteToFilesystem(e))?;
        }

        Ok(Self {
            url: url.to_string(),
            data: None,
            json_data: None,
        })
    }

    fn open(&'a mut self) -> Result<T, Error> {
        let data =
            fs::read_to_string(self.url.as_str()).map_err(|e| Error::CannotAccessFilesystem(e))?;
        self.data = Some(data);

        let json_data = json::from_str::<'a, T>(self.data.as_ref().unwrap().as_str())
            .map_err(|e| Error::FailedToParseJson(e))?;

        self.json_data = Some(json_data.clone());

        Ok(json_data)
    }

    /// Reads data from provided url. Data is cached after first read.
    pub fn read(&'a mut self) -> Result<T, Error> {
        if let Some(t) = self.json_data.clone() {
            return Ok(t);
        }
        Ok(self.open()?)
    }

    pub fn write(&'a mut self, new_data: T) -> Result<(), Error> {
        let s = json::to_string(&new_data).map_err(|e| Error::FailedToSerializeIntoJson(e))?;
        fs::write(self.url.as_str(), s).map_err(|e| Error::CannotWriteToFilesystem(e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct JsonObj {
        string: String,
        num: i32,
        bool: bool,
    }

    // post-test cleanup
    impl<T> Drop for Client<T> {
        fn drop(&mut self) {
            // if this function fails, it's likely because the file didn't exist, which is ok
            fs::remove_file(self.url.as_str()).unwrap_or({});
        }
    }

    #[test]
    fn test_init_creates_file() {
        let url = "test_init_creates_file.json";
        let _client = Client::<JsonObj>::init(url).unwrap();
        assert!(fs::try_exists(url).unwrap());
    }

    #[test]
    fn test_init_doesnt_creates_file_if_already_existing() {
        let url = "test_init_doesnt_creates_file_if_already_existing.json";
        let message = "hi";
        fs::write(url, message).unwrap();
        assert!(fs::try_exists(url).unwrap());
        let _client = Client::<JsonObj>::init(url).unwrap();
        assert_eq!(fs::read_to_string(url).unwrap(), message);
    }

    #[test]
    fn test_read_json() {
        let url = "test_read_json.json";
        let test_string = "hi there";
        let test_num = 8;
        let test_bool = true;
        let json = format!(
            "\
{{
    \"string\": \"{test_string}\",
    \"num\": {test_num},
    \"bool\": {test_bool}
}}\
            "
        );
        fs::write(url, json).unwrap();
        assert!(fs::try_exists(url).unwrap());
        let mut client = Client::<JsonObj>::init(url).unwrap();
        let results = client.read().unwrap();
        assert_eq!(results.string, test_string);
        assert_eq!(results.num, test_num);
        assert_eq!(results.bool, test_bool);
    }

    #[test]
    fn test_write_json() {
        let url = "test_write_json.json";
        let test_string = "hi there";
        let test_num = 8;
        let test_bool = true;
        let obj = JsonObj {
            string: test_string.to_string(),
            num: test_num,
            bool: test_bool,
        };
        let expected_json = format!(
            "\
{{
    \"string\": \"{test_string}\",
    \"num\": {test_num},
    \"bool\": {test_bool}
}}\
            "
        )
        .replace(" ", "")
        .replace("\n", "");
        let mut client = Client::<JsonObj>::init(url).unwrap();
        client.write(obj).unwrap();
        let results = fs::read_to_string(url)
            .unwrap()
            .replace(" ", "")
            .replace("\n", "");
        assert_eq!(results, expected_json);
    }
}
