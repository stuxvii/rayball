use std::{collections::HashMap, error::Error};
use url::Url;

pub fn parse_code(clip: String) -> Result<std::string::String, Box<dyn Error>> {
    let parsed_url_result = Url::parse(&clip);
    let parsed_url: Url;
    match parsed_url_result {
        Ok(u) => {
            parsed_url = u;
        },
        Err(e) => {
            return Err(e.into())
        }
    };
    let query_map: HashMap<String, String> = parsed_url
        .query_pairs()
        .into_owned()
        .collect();
    if let Some(c) = query_map.get("c") {
        return Ok(c.to_string());
    } else {
        return Err("Didn't find code in URL".into())
    }
}