use std::sync::Arc;

use regex::Regex;
use tokio::sync::Mutex;
// use lazy_static::lazy_static;

use crate::prisma::{self, PrismaClient};

pub fn is_comic_page(page: &str) -> bool {
    page.contains("Thêm vào bookmark")
}

pub async fn parse_comic_page(
    page: &str,
    url: &str,
    client: Arc<Mutex<PrismaClient>>,
) -> Option<Vec<String>> {
    let client = client.lock().await;
    let mut result = Vec::new();
    // fetch all urls from page
    let db_comic = {
        let tmp = client
            .comic()
            .find_first(vec![prisma::comic::url::equals(url.to_string())])
            .exec()
            .await;
        if tmp.is_err() {
            return None;
        }
        let tmp = tmp.unwrap();
        if tmp.is_none() {
            if let Ok(comic) = client.comic().create(url.to_string(), vec![]).exec().await {
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
        update_field.push(prisma::comic::name::set(title));
    }
    let comic_exec = {
        let tmp = client
            .comic()
            .update(
                prisma::comic::UniqueWhereParam::UrlEquals(db_comic.url),
                update_field,
            )
            .exec()
            .await;
        if tmp.is_err() {
            return None;
        }
        tmp.unwrap()
    };

    let comic_id = comic_exec.id.to_string();

    let mut chapters = vec![];
    let mut update_chapters = vec![];
    // regex for a chapter
    let  chapter_regex = Regex::new(r#"<p\s+id="chapter-(\d+)">\s+<span\s+class="title">\s+<a\s+id="\w+_\d+"\s+href="(.+)"\s+title=".+>(.+)<\/a>\s+<\/span>\s+<span\s+class="publishedDate">(.+)<\/span>"#).unwrap();
    let mut i = 0;
    // get all chapters count
    let chapter_count = chapter_regex.captures_iter(page).count();
    for cap in chapter_regex.captures_iter(page) {
        // let id = cap[1].to_string();
        let mut url = format!("https://blogtruyenmoi.com{}", cap[2].to_string())
            .trim()
            .to_string();
        if url.contains("\" title=") {
            url = url.replace("\" title=\"", "").trim().to_string();
        }
        let title = cap[3].to_string().trim().to_string();
        let date = cap[4].to_string();
        println!("found chapter url {}", url);
        {
            let tmp = client
                .chapter()
                .find_first(vec![prisma::chapter::url::equals(url.to_string())])
                .exec()
                .await;
            if tmp.is_err() {
                continue;
            }
            let tmp = tmp.unwrap();
            if tmp.is_some() {
                let index = chapter_count.clone() as i32 - i;
                if tmp.unwrap().index != index {
                    update_chapters.push(client.chapter().update_many(
                        vec![prisma::chapter::url::equals(url.to_string())],
                        vec![prisma::chapter::index::set(index)],
                    ));
                } else {
                    println!("chapter {} already exists", url);
                }
                result.push(url.to_string());
                i += 1;
                continue;
            }
        }
        chapters.push(prisma::chapter::create_unchecked(
            title.to_string(),
            url.to_string(),
            comic_id.to_string(),
            date.to_string(),
            vec![prisma::chapter::index::set(i)],
        ));
        result.push(url.to_string());
        i += 1;
    }
    if client._batch(update_chapters).await.is_err() {
        return None;
    };
    if (client.chapter().create_many(chapters))
        .exec()
        .await
        .is_err()
    {
        return None;
    };

    Some(result)
}

pub async fn parse_chapter_page(
    url: &str,
    html: &str,
    client: &PrismaClient,
) -> Option<Vec<String>> {
    {
        let tmp = client
            .urls()
            .find_first(vec![prisma::urls::url::equals(url.to_string().clone())])
            .exec()
            .await;
        if tmp.is_err() {
            return None;
        }
        let tmp = tmp.unwrap();
        if tmp.is_none() {
            client
                .urls()
                .create(
                    url.to_string().clone(),
                    vec![
                        prisma::urls::fetched::set(false),
                        prisma::urls::fetching::set(true),
                    ],
                )
                .exec()
                .await
                .unwrap();
        } else if tmp.clone().unwrap().fetched == false && tmp.unwrap().fetching == false {
            client
                .urls()
                .update_many(
                    vec![prisma::urls::url::equals(url.to_string())],
                    vec![
                        prisma::urls::fetched::set(false),
                        prisma::urls::fetching::set(true),
                    ],
                )
                .exec()
                .await
                .unwrap();
        }
    }
    println!("fetching chapter {}", url);
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
    client
        .chapter()
        .update_many(
            vec![prisma::chapter::url::equals(url.to_string())],
            vec![prisma::chapter::images::set(images_urls)],
        )
        .exec()
        .await
        .unwrap();
    client
        .urls()
        .update_many(
            vec![prisma::urls::url::equals(url.to_string())],
            vec![
                prisma::urls::fetched::set(true),
                prisma::urls::fetching::set(false),
            ],
        )
        .exec()
        .await
        .unwrap();
    Some(result)
}
