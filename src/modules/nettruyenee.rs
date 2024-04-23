use std::{collections::HashMap, sync::Arc};

use regex::Regex;
use scraper::{Html, Selector};
use serde_json::json;
use tokio::sync::Mutex;
// use lazy_static::lazy_static;

use crate::db::{ChapterUpdateField, DbUtils, UpdateComicDocField};

use super::NETTRUYEN_HOSTS;

pub fn is_comic_page(url: &str, _page: &str) -> bool {
    let regex = Regex::new(&format!(
        r#"https:\/\/({})\/truyen-tranh\/(.+)"#,
        NETTRUYEN_HOSTS.join("|")
    ))
    .unwrap();
    regex.is_match(url)
}

pub async fn parse_comic_page(
    page: &str,
    url: &str,
    client: Arc<Mutex<DbUtils>>,
) -> Option<Vec<String>> {
    log::info!("parsing comic page {}", url);
    let client = client.lock().await;
    let mut result = Vec::new();
    // fetch all urls from page
    let db_comic = {
        // let tmp = client
        //     .comic()
        //     .find_unique(prisma::comic::UniqueWhereParam::UrlEquals(url.to_string()))
        //     .exec()
        //     .await;
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
    let title_regex = Regex::new(r#"<title>\s+(.*?)\s+<\/title>"#).unwrap();
    let title = {
        let tmp = title_regex.captures(page);
        if tmp.is_none() {
            return None;
        }
        tmp.unwrap()[1].to_string() + " - NetTruyenEe.com"
    };
    if !db_comic.name.eq(&title) {
        update_field.push(UpdateComicDocField::Name(title));
    }
    let (_, comic_info) = parse_net_truyen_html_page(&page.to_string());
    for f in comic_info {
        update_field.push(f.clone());
    }
    let comic_exec = {
        // let tmp = client
        //     .comic()
        //     .update(
        //         prisma::comic::UniqueWhereParam::UrlEquals(db_comic.url),
        //         update_field,
        //     )
        //     .exec()
        //     .await;
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
    let chapter_regex =
        Regex::new(r#"<a\s+href="([^"|"]+)"\s+data-id="([^"]+)">([^\/]+)<\/a>"#).unwrap();
    let mut i = 0;
    // get all chapters count
    let chapter_count = chapter_regex.captures_iter(page).count();
    for cap in chapter_regex.captures_iter(page) {
        let index = chapter_count.clone() as i32 - i;
        // let id = cap[1].to_string();
        let mut url = cap[1].trim().to_string();
        if !is_chapter_page(&url, "") {
            continue;
        };
        if url.contains("\" title=") {
            url = url.replace("\" title=\"", "").trim().to_string();
        }
        let title = cap[3].to_string().trim().to_string();
        log::info!("found chapter url {}", url);
        {
            // let tmp = client
            //     .chapter()
            //     .find_unique(prisma::chapter::UniqueWhereParam::UrlEquals(
            //         url.to_string(),
            //     ))
            //     .exec()
            //     .await;
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
        // chapters.push(prisma::chapter::create_unchecked(
        //     title.to_string(),
        //     url.to_string(),
        //     comic_id.to_string(),
        //     "".to_owned(),
        //     vec![prisma::chapter::index::set(index)],
        // ));
        chapters.push(client.create_empty_chapter_dafter(
            title.clone(),
            url.clone(),
            "".to_string(),
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
    let created_date = {
        let created_date_re = Regex::new(r#"<i>\[Cập nhật lúc:\s+(.+)\]<\/i>"#).unwrap();
        let tmp = created_date_re.captures(html);
        if tmp.is_none() {
            return None;
        }
        tmp.unwrap()[1].to_string()
    };
    let result = Vec::new();

    let images_url_regex = Regex::new(r#"<img\s+alt="[^"]+"\s+data-index="[^"]+"\s+src="([^"]+)"\s+data-original="[^"]+" data-cdn="([^"]+)"\s+\/>"#).unwrap();
    let mut images_urls = vec![];
    for cap in images_url_regex.captures_iter(&html) {
        let url = format!("{}{}", "https:".to_string(), cap[1].to_string());
        let cdn = format!("{}{}", "https:".to_string(), cap[2].to_string());
        images_urls.push(serde_json::json!({
            "url": url,
            "cdn": cdn
        }));
    }
    // client
    //     .chapter()
    //     .update_many(
    //         vec![prisma::chapter::url::equals(url.to_string())],
    //         vec![
    //             prisma::chapter::server_image::set(images_urls),
    //             prisma::chapter::created_date::set(created_date),
    //         ],
    //     )
    //     .exec()
    //     .await
    //     .unwrap();
    match client
        .update_chapter_by_url(
            url.to_string(),
            vec![
                ChapterUpdateField::ServerImage(images_urls),
                ChapterUpdateField::CreatedDate(created_date),
            ],
        )
        .await
    {
        Ok(_) => {}
        Err(_) => return None,
    }
    Some(result)
}

pub fn is_chapter_page(url: &str, _html: &str) -> bool {
    let re = Regex::new(
        format!(
            r#"https:\/\/({})\/truyen-tranh\/(.+)\/chap-(.+)\/(.+)"#,
            NETTRUYEN_HOSTS.join("|")
        )
        .as_str(),
    )
    .unwrap();
    if re.is_match(url) {
        log::info!("{} is chapter page", url);
        return true;
    } else {
        return false;
    }
}

pub fn parse_net_truyen_html_page(
    html: &str,
) -> (HashMap<String, serde_json::Value>, Vec<UpdateComicDocField>) {
    let mut result = HashMap::new();
    let mut update_data = vec![UpdateComicDocField::PythonFetchInfo(true)];
    let document = Html::parse_document(html);

    // fetch content, thumbnail
    // title fetch before
    let content_selector = Selector::parse("#item-detail > div.detail-content > p").unwrap();
    let content = document.select(&content_selector).next();
    if let Some(content) = content {
        let content = content.inner_html().trim().to_string();
        update_data.push(UpdateComicDocField::Content(Some(content.to_string())));
        result.insert("content".to_string(), json!(content.to_string()));
    }
    let thumbnail_selector =
        Selector::parse("#item-detail > div.detail-info > div > div.col-xs-4.col-image > img")
            .unwrap();
    let thumbnail = document.select(&thumbnail_selector).next();
    if let Some(thumbnail) = thumbnail {
        let thumbnail = thumbnail.value().attr("src").unwrap().trim().to_string();
        update_data.push(UpdateComicDocField::ThumbnailUrl(Some(
            thumbnail.to_string(),
        )));
        result.insert("thumbnail".to_string(), json!(thumbnail.to_string()));
    }

    let another_name_selector = Selector::parse(
        "#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.othername.row > h2",
    )
    .unwrap();
    let another_name = document.select(&another_name_selector).next();
    if let Some(another_name) = another_name {
        let another_name = another_name
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .split(";")
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        update_data.push(UpdateComicDocField::AnotherName(another_name.clone()));
        result.insert("another_name".to_string(), json!(another_name.clone()));
    }

    let genre_selector = Selector::parse(
        "#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.kind.row > p.col-xs-8"
    ).unwrap();
    let genre = document.select(&genre_selector).next();
    if let Some(p) = genre {
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

    let author_selector = Selector::parse(
        "#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.author.row > p.col-xs-8"
    ).unwrap();
    let author = document.select(&author_selector).next();
    if let Some(p) = author {
        let author_list_selector = Selector::parse("a").unwrap();
        let author_list = p.select(&author_list_selector);
        let mut author = HashMap::new();
        if author_list.clone().count() > 0 {
            for a in author_list {
                let child_text = a.text().collect::<String>();
                let url = a.value().attr("href").unwrap();
                author.insert(child_text, url.to_string());
            }
        } else {
            for a in p.text().collect::<String>().trim().split(";") {
                author.insert(a.to_string(), "".to_string());
            }
        }
        update_data.push(UpdateComicDocField::Author(json!(author.clone())));
        result.insert("author".to_string(), json!(author));
    }

    (result, update_data)
}

mod test {
    #[allow(unused_imports)]
    use serde_json::json;

    #[tokio::test]
    async fn test_parse_net_truyen_html_page() {
        let client = reqwest::Client::new();
        let html = client
            .get("https://nettruyenee.com/truyen-tranh/samurai-kiem-tre-107900")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .unwrap();
        let html = html.text().await.unwrap();
        let (result, _) = super::parse_net_truyen_html_page(&html);
        // print out result
        log::info!("{:?}", json!(result));
    }
}
