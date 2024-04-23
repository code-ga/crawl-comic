use regex::Regex;

pub fn get_host(url: &str) -> Option<String> {
    let re = Regex::new(r#"https?://([^/]+)"#).unwrap();
    let cap = re.captures(url);
    if cap.is_none() {
        return None;
    }
    return Some(cap.unwrap()[1].to_string());
}
