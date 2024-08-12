use std::sync::Arc;

use prisma_client_rust::chrono::{self, Utc};
use regex::Regex;
use tokio::sync::Mutex;
// use lazy_static::lazy_static;

use crate::db::{ChapterUpdateField, DbUtils, UpdateComicDocField};

pub fn is_comic_page(url: &str, _page: &str) -> bool {
    url.contains("https://alyasometimeshidesherfeelings.com/")
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
        let title = cap[2].to_string().trim().to_string();
        let date = {
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

pub async fn parse_chapter_page(url: &str, html: &str, client: &DbUtils) -> Option<Vec<String>> {
    log::info!("fetching chapter {}", url);
    let result = Vec::new();

    let images_url_regex = Regex::new(r#"src="([^"]+)"#).unwrap();
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

    // fetch content, thumbnail
    // title fetch before
    let content_selector = Selector::parse(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.detail > div.content",
    )
    .unwrap();
    let content = document.select(&content_selector).next();
    if let Some(content) = content {
        let content = content.inner_html().trim().to_string();
        update_data.push(UpdateComicDocField::Content(Some(content.to_string())));
        result.insert("content".to_string(), json!(content.to_string()));
    }
    let thumbnail_selector = Selector::parse(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.thumbnail > img",
    )
    .unwrap();
    let thumbnail = document.select(&thumbnail_selector).next();
    if let Some(thumbnail) = thumbnail {
        let thumbnail = thumbnail.value().attr("src").unwrap().trim().to_string();
        update_data.push(UpdateComicDocField::ThumbnailUrl(Some(
            thumbnail.to_string(),
        )));
        result.insert("thumbnail".to_string(), json!(thumbnail.to_string()));
    }

    // fetch another information
    let description_selector = Selector::parse(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.description",
    )
    .unwrap();
    let description = document.select(&description_selector).next();
    if let Some(description) = description {
        let p_selector = Selector::parse("p").unwrap();
        let ps = description.select(&p_selector);
        for p in ps {
            let child_text = p.text().collect::<String>().trim().to_string();
            if child_text.starts_with("Tên khác:") {
                // remove prefix "Tên khác:" from string
                let another_name = {
                    let temp = child_text.to_string();
                    let split_child_text = temp.split(":").collect::<Vec<&str>>();
                    split_child_text.clone()[1..]
                        .join(":")
                        .split(",")
                        .map(|x| x.trim().to_string())
                        .collect::<Vec<_>>()
                        .clone()
                };
                update_data.push(UpdateComicDocField::AnotherName(another_name.clone()));
                result.insert("anotherName".to_string(), json!(another_name));
            } else if child_text.starts_with("Tác giả:") {
                let author_list_selector = Selector::parse("a").unwrap();
                let author_list = p.select(&author_list_selector);
                let mut author = HashMap::new();
                for a in author_list {
                    let child_text = a.text().collect::<String>();
                    let url = a.value().attr("href").unwrap();
                    author.insert(child_text, url.to_string());
                }
                update_data.push(UpdateComicDocField::Author(json!(author.clone())));
                result.insert("author".to_string(), json!(author));
            } else if child_text.starts_with("Nguồn:") {
                let source_list_selector = Selector::parse("a").unwrap();
                let source_list = p.select(&source_list_selector);
                let mut source = HashMap::new();
                for a in source_list {
                    let child_text = a.text().collect::<String>();
                    let url = a.value().attr("href").unwrap();
                    source.insert(child_text, url.to_string());
                }
                update_data.push(UpdateComicDocField::Source(json!(source.clone())));
                result.insert("source".to_string(), json!(source));
            } else if child_text.starts_with("Nhóm dịch:") {
                let translator_link_selector = Selector::parse("a").unwrap();
                let translator_link = p.select(&translator_link_selector);
                let mut translator = HashMap::new();
                for a in translator_link {
                    let child_text = a.text().collect::<String>();
                    let url = a.value().attr("href").unwrap();
                    translator.insert(child_text, url.to_string());
                }
                update_data.push(UpdateComicDocField::TranslatorTeam(json!(
                    translator.clone()
                )));
                result.insert("translatorTeam".to_string(), json!(translator));
            } else if child_text.starts_with("Đăng bởi:") {
                let posted_by_list_selector = Selector::parse("a").unwrap();
                let posted_by_list = p.select(&posted_by_list_selector);
                let mut posted_by = HashMap::new();
                for a in posted_by_list {
                    let child_text = a.text().collect::<String>();
                    let url = a.value().attr("href").unwrap();
                    posted_by.insert(child_text, url.to_string());
                }
                update_data.push(UpdateComicDocField::PostedBy(json!(posted_by.clone())));
                result.insert("postedBy".to_string(), json!(posted_by));
                let status_selector = Selector::parse("span").unwrap();
                let status = p.select(&status_selector).next();
                if let Some(status) = status {
                    let status = status.text().collect::<String>();
                    update_data.push(UpdateComicDocField::Status(status.trim().to_string()));
                    result.insert("status".to_string(), json!(status.trim().to_string()));
                }
            } else if child_text.starts_with("Thể loại:") {
                let genre_list_selector = Selector::parse("a").unwrap();
                let genre_list = p.select(&genre_list_selector);
                let mut genre = HashMap::new();
                for a in genre_list {
                    let child_text = a.text().collect::<String>();
                    let url = a.value().attr("href").unwrap();
                    genre.insert(child_text, url.to_string());
                }
                update_data.push(UpdateComicDocField::Genre(json!(genre.clone())));
                result.insert("genre".to_string(), json!(genre));
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
            .get("https://blogtruyenmoi.com/34464/kiyota-san-muon-bi-vay-ban")
            .header("User-Agent", "Mozilla/5.0")
            .header("Referrer", "https://blogtruyenmoi.com/")
            .send()
            .await
            .unwrap();
        let html = html.text().await.unwrap();
        let (result, _) = super::parse_alyasometimeshidesherfeelings_moi_html_page(&html);
        // print out result
        log::info!("{:?}", json!(result));
    }
}
