use std::{sync::Arc, vec};

use prisma_client_rust::chrono::{self, Utc};
use regex::Regex;
use tokio::sync::Mutex;
// use lazy_static::lazy_static;

use crate::db::{ChapterUpdateField, DbUtils, UpdateComicDocField};

pub fn is_comic_page(url: &str, _page: &str) -> bool {
    url.eq("https://alyasometimeshidesherfeelings.com/")
}

pub async fn parse_comic_page(
    page: &str,
    url: &str,
    client: Arc<Mutex<DbUtils>>,
) -> Option<Vec<String>> {
    let client = client.lock().await;
    let mut result = Vec::new();
    // fetch all urls from page
    let db_comic = {
        let tmp = client.comic_by_url(url.to_string()).await;
        if tmp.is_err() {
            return None;
        }
        let tmp = tmp.unwrap();
        if tmp.is_none() {
            if let Ok(comic) = client.create_empty_comic(url.to_string()).await {
                comic
            } else {
                return None;
            }
        } else {
            tmp.unwrap()
        }
    };
    let mut update_field = vec![];
    let title_regex = Regex::new(r#"<title>(.*?)</title>"#).unwrap();
    let title = {
        let tmp = title_regex.captures(page);
        if tmp.is_none() {
            return None;
        }
        tmp.unwrap()[1].to_string()
    };
    if !db_comic.name.eq(&title) {
        update_field.push(UpdateComicDocField::Name(title));
    }
    let (_, comic_info) = parse_alyasometimeshidesherfeelings_moi_html_page(&page.to_string());
    for f in comic_info {
        update_field.push(f.clone());
    }
    let comic_exec = {
        //     let tmp = client
        //         .comic()
        //         .update(
        //             prisma::comic::UniqueWhereParam::UrlEquals(db_comic.url),
        //             update_field,
        //         )
        //         .exec()
        //         .await;
        let tmp = client.update_comic_by_url(db_comic.url, update_field).await;
        if tmp.is_err() {
            return None;
        }
        tmp.unwrap()
    };

    let comic_id = comic_exec.id.to_string();

    let mut chapters = vec![];
    let mut update_chapters = vec![];
    // regex for a chapter
    let  chapter_regex = 
        Regex::new(r#"<a\s+href="([^"|"]+)"><span>([^>]+)<\/span><span\s+class="chapter-date">([^>]+)<\/span><\/a>"#)
        .unwrap();
    let mut i = 0;
    // get all chapters count
    let chapter_count = chapter_regex.captures_iter(page).count();
    for cap in chapter_regex.captures_iter(page) {
        let url = cap[1].to_string()
            .trim()
            .to_string();
        if url.ends_with("/feed/") {
            log::info!("found feed url {}", url);
            continue;
        }
        let title = cap[2].to_string().trim().to_string();
        let date = {
            if cap[3].is_empty() || cap[3].contains("🔥") {
                continue;
            }
            let tmp = cap[3].to_string().replace("ago", "").trim().to_string();
            let mut time = Utc::now();
            if tmp.contains("year"){
                let num_year = tmp.split(" ").collect::<Vec<&str>>()[0].parse::<i32>().unwrap();
                time = time.checked_sub_months(chrono::Months::new((12*num_year).try_into().unwrap())).unwrap();
            }else if tmp.contains("month"){ 
                let num_month = tmp.split(" ").collect::<Vec<&str>>()[0].parse::<i32>().unwrap();
                time = time.checked_sub_months(chrono::Months::new(num_month.try_into().unwrap())).unwrap();
            }else if tmp.contains("week"){
                let num_week = tmp.split(" ").collect::<Vec<&str>>()[0].parse::<i32>().unwrap();
                time = time.checked_sub_days(chrono::Days::new((7*num_week).try_into().unwrap())).unwrap();
            }else if tmp.contains("day"){ 
                let num_day = tmp.split(" ").collect::<Vec<&str>>()[0].parse::<i32>().unwrap();
                time = time.checked_sub_days(chrono::Days::new(num_day.try_into().unwrap())).unwrap();
            }
            time.to_string()
        };
        log::info!("found chapter url {}", url);
        let index = chapter_count.clone() as i32 - i;
        {
            let tmp = client.chapter_by_url(url.to_string()).await;
            if tmp.is_err() {
                continue;
            }
            let tmp = tmp.unwrap();
            if tmp.is_some() {
                if tmp.unwrap().index != index {
                    // update_chapters.push(client.chapter().update_many(
                    //     vec![prisma::chapter::url::equals(url.to_string())],
                    //     vec![prisma::chapter::index::set(index)],
                    // ));
                    update_chapters.push(client.daft_update_chapter_by_url(
                        url.to_string(),
                        vec![ChapterUpdateField::Index(index.clone())],
                    ));
                } else {
                    log::info!("chapter {} already exists", url);
                }
                result.push(url.to_string());
                i += 1;
                continue;
            }
        }
        chapters.push(client.create_empty_chapter_dafter(
            title.clone(),
            url.clone(),
            date.clone(),
            comic_id.clone(),
            Some(index.clone()),
        ));
        result.push(url.to_string());
        i += 1;
    }
    if client._batch(update_chapters).await.is_err() {
        return None;
    };
    if client.create_many_chapters(chapters).await.is_err() {
        return None;
    };

    Some(result)
}

// const IMAGES_EXTENSIONS = ["jpg", "jpeg", "png", "gif","webp"];

pub async fn parse_chapter_page(url: &str, html: &str, client: &DbUtils) -> Option<Vec<String>> {
    log::info!("fetching chapter {}", url);
    let result = Vec::new();

    let images_url_regex = Regex::new(r#"<img.+src="([^"]+)"#).unwrap();
    let mut images_urls = vec![];
    for cap in images_url_regex.captures_iter(&html) {
        let url = cap[1].to_string();
        if !url.starts_with("http") {
            continue;
        }
        images_urls.push(url);
    }
    // client
    //     .chapter()
    //     .update_many(
    //         vec![prisma::chapter::url::equals(url.to_string())],
    //         vec![prisma::chapter::images::set(images_urls)],
    //     )
    //     .exec()
    //     .await
    //     .unwrap();
    if images_urls.is_empty() {
        return None;
    }
    match client
        .update_chapter_by_url(
            url.to_string(),
            vec![ChapterUpdateField::Images(images_urls)],
        )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            log::error!("failed to update chapter {}: {}", url, e);
            return None;
        }
    };
    Some(result)
}

pub fn is_chapter_page(url: &str, _html: &str) -> bool {
    url.starts_with("https://alyasometimeshidesherfeelings.com/manga/")
}

use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;

pub fn parse_alyasometimeshidesherfeelings_moi_html_page(
    html: &str,
) -> (HashMap<String, serde_json::Value>, Vec<UpdateComicDocField>) {
    let mut result = HashMap::new();
    let mut update_data = vec![UpdateComicDocField::PythonFetchInfo(true)];
    let document = Html::parse_document(html);

    let thumbnail_selector = Selector::parse("#content > div > div.left-column > img").unwrap();
    let thumbnail = document.select(&thumbnail_selector).next();
    if let Some(thumbnail) = thumbnail {
        let thumbnail = thumbnail.value().attr("src").unwrap();
        update_data.push(UpdateComicDocField::ThumbnailUrl(Some(thumbnail.to_string())));
        result.insert("thumbnail".to_string(), json!(thumbnail));
    }

    let comic_info_list_selector = Selector::parse(
        "#content > div > div.right-column > div > ul").unwrap();
    let comic_info_list = document.select(&comic_info_list_selector).next();
    if let Some(comic_info_list) = comic_info_list {
        let li_selector = Selector::parse("li").unwrap();
        let li_list = comic_info_list.select(&li_selector);
        for li in li_list {
            let child_text = li.text().collect::<String>();
            if child_text.starts_with("Alternate Name(s):") {
                let another_name = vec![child_text.replace("Alternate Name(s):", "").trim().to_string()];    
                update_data.push(UpdateComicDocField::AnotherName(another_name.clone()));
                result.insert("another_name".to_string(), json!(another_name));
            }
            if child_text.starts_with("Author(s):"){
                let mut author_list = HashMap::new();
                for author in child_text.replace("Author(s):", "").trim().to_string().split(",") {
                    let author_name = author.trim().to_string();
                    author_list.insert(author_name, "".to_string());
                }
                update_data.push(UpdateComicDocField::Author(json!(author_list)));
                result.insert("author".to_string(), json!(author_list));
            }
            if child_text.starts_with("Genre(s):") {
                let mut genre = HashMap::new();
                for genre_name in child_text.replace("Genre(s):", "").trim().to_string().split(",") {
                    let genre_name = genre_name.trim().to_string();
                    genre.insert(genre_name, "".to_string());
                }
                update_data.push(UpdateComicDocField::Genre(json!(genre)));
                result.insert("genre".to_string(), json!(genre));
            }
            if child_text.starts_with("Status:") {
                let status = child_text.replace("Status:", "").trim().to_string();
                update_data.push(UpdateComicDocField::Status(status.clone()));
                result.insert("status".to_string(), json!(status));
            }
            if child_text.starts_with("Description:") {
                let description = child_text.replace("Description:", "").trim().to_string();
                update_data.push(UpdateComicDocField::Content(Some(description.clone())));
                result.insert("description".to_string(), json!(description));
            }
        }
    }

    (result, update_data)
}

mod test {
    #[allow(unused_imports)]
    use serde_json::json;

    #[tokio::test]
    async fn test_parse_alyasometimeshidesherfeelings_moi_html_page() {
        let client = reqwest::Client::new();
        let html = client
            .get("https://alyasometimeshidesherfeelings.com/manga/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .unwrap();
        let html = html.text().await.unwrap();
        let (result, _) = super::parse_alyasometimeshidesherfeelings_moi_html_page(&html);
        // print out result
        log::info!("{:?}", json!(result));
    }
}
