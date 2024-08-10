use std::{collections::HashMap, sync::Arc};

use prisma_client_rust::chrono::Utc;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::json;
use tokio::sync::Mutex;
// use lazy_static::lazy_static;

use crate::{
    db::{ChapterUpdateField, DbUtils, UpdateComicDocField},
    util,
};

pub fn is_comic_page(url: &str, _page: &str) -> bool {
    url.contains("https://mangajuice.com/manga")
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
    let title_regex = Regex::new(r#"<title>(.*?)<\/title>"#).unwrap();
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
    let (_, comic_info) = parse_mangajuice_html_page(&page.to_string());
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
        Regex::new(r#"<a\s+href="([^"|"]+)"\s+class="title">([^\/]+)<\/a>"#).unwrap();
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
        if url.ends_with("/feed/") {
            log::info!("found feed url {}", url);
            continue;
        }
        let title = cap[2].to_string().trim().to_string();
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
            Utc::now().to_string(),
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

    let images_url_regex = Regex::new(r#"<img.+src="([^"]+)"#).unwrap();
    let mut images_urls = vec![];
    for cap in images_url_regex.captures_iter(&html) {
        let url = cap[1].to_string();
        // log::info!("found image url {}. Uploading...", url);
        // request to get image
        // let image = if let Ok(data) = reqwest::get(&url).await {
        //     log::info!("request image url {}", url);
        //     // get bytes
        //     if let Ok(byte) = data.bytes().await {
        //         byte
        //     } else {
        //         images_urls.push(url);
        //         continue;
        //     }
        // } else {
        //     log::info!("failed to request image url {}", url);
        //     images_urls.push(url);
        //     continue;
        // };
        // // upload to guilded
        // let cdn_url = util::upload_image_to_guilded(image.to_vec()).await;
        // if let Ok(url) = cdn_url {
        //     log::info!("uploaded image url {}", url);
        //     images_urls.push(url);
        // } else {
        //     log::info!("failed to upload image url {}", url);
        //     images_urls.push(url);
        // }

        images_urls.push(url);
    }
    if images_urls.len() == 0 {
        log::info!("no image found in chapter {}", url);
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
    Some(Vec::new())
}

pub fn is_chapter_page(url: &str, _html: &str) -> bool {
    url.starts_with("https://mangajuice.com/chapter")
}

pub fn parse_mangajuice_html_page(
    html: &str,
) -> (HashMap<String, serde_json::Value>, Vec<UpdateComicDocField>) {
    let mut result = HashMap::new();
    let mut update_data = vec![UpdateComicDocField::PythonFetchInfo(true)];
    let document = Html::parse_document(html);

    // fetch content, thumbnail
    // title fetch before
    let content_selector = Selector::parse("#hidden > p").unwrap();
    let content = document.select(&content_selector).next();
    if let Some(content) = content {
        let content = content.inner_html().trim().to_string();
        update_data.push(UpdateComicDocField::Content(Some(content.to_string())));
        result.insert("content".to_string(), json!(content.to_string()));
    }
    let thumbnail_selector = Selector::parse("div.manga-cover > a > img").unwrap();
    let thumbnail = document.select(&thumbnail_selector).next();
    if let Some(thumbnail) = thumbnail {
        let thumbnail = thumbnail.value().attr("src").unwrap().trim().to_string();
        update_data.push(UpdateComicDocField::ThumbnailUrl(Some(
            thumbnail.to_string(),
        )));
        result.insert("thumbnail".to_string(), json!(thumbnail.to_string()));
    }

    let another_name_selector = Selector::parse(
        "div.entry-content > div > div.row > div.col-md-10 > figure > table > tbody > tr:nth-child(3) > td:nth-child(2)",
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
        "div.entry-content > div > div.row > div.col-md-10 > figure > table > tbody > tr:nth-child(5) > td:nth-child(2)"
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
        "div.entry-content > div > div.row > div.col-md-10 > figure > table > tbody > tr:nth-child(4) > td:nth-child(2)"
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
            .get("https://mangajuice.com/manga/ragna-crimson/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .unwrap();
        let html = html.text().await.unwrap();
        let (result, _) = super::parse_mangajuice_html_page(&html);
        // print out result
        log::info!("{:?}", json!(result));
    }
}
