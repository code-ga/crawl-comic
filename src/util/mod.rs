use rand::Rng;
use regex::Regex;
pub mod fetch_page;

#[allow(dead_code)]
static UPLOAD_URL: &str =
    "https://media.guilded.gg/media/upload?dynamicMediaTypeId=ContentMediaGenericFiles";

pub fn get_host(url: &str) -> Option<String> {
    let re = Regex::new(r#"https?://([^/]+)"#).unwrap();
    let cap = re.captures(url);
    if cap.is_none() {
        return None;
    }
    return Some(cap.unwrap()[1].to_string());
}

pub async fn upload_image_to_guilded(
    file: Vec<u8>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let bot_token = std::env::var("GUILDED_BOT_TOKEN")?;
    let wait_time = rand::thread_rng().gen_range(1..5);
    tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
    let client = reqwest::Client::new();
    let body = reqwest::multipart::Form::new().part("file", reqwest::multipart::Part::bytes(file));
    let resp = client
        .post(UPLOAD_URL)
        .bearer_auth(bot_token)
        .multipart(body)
        .send()
        .await?;
    if resp.status().is_success() {
        let resp = resp.json::<serde_json::Value>().await?;
        let regex = Regex::new(r#"https://.*?.amazonaws\.com/www\.guilded\.gg/"#).unwrap();
        // regex replace with https://cdn.gilcdn.com/
        let resp = regex.replace_all(&resp["url"].as_str().unwrap(), "https://cdn.gilcdn.com/");
        return Ok(resp.to_string());
    }
    Err("failed to upload image".into())
}
