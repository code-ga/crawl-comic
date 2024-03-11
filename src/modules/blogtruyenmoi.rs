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
    // {
    //     let tmp = client
    //         .comic()
    //         .find_first(vec![prisma::comic::url::equals(url.to_string())])
    //         .exec()
    //         .await;
    //     if tmp.is_err() {
    //         return None;
    //     }
    //     if tmp.unwrap().is_some() {
    //         return Some(Vec::new());
    //     }
    // }
    // println!("fetching comic page {}", url);
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
    // regex for a chapter
    let  chapter_regex = Regex::new(r#"<p\s+id="chapter-(\d+)">\s+<span\s+class="title">\s+<a\s+id="\w+_\d+"\s+href="(.+)"\s+title=".+>(.+)<\/a>\s+<\/span>\s+<span\s+class="publishedDate">(.+)<\/span>"#).unwrap();
    for cap in chapter_regex.captures_iter(page) {
        // println!("{} {}", cap[1].to_string(), cap[2].to_string());
        // let wait = rand::thread_rng().gen_range(3..5);
        // tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
        // let id = cap[1].to_string();
        let mut url = format!("https://blogtruyenmoi.com{}", cap[2].to_string())
            .trim()
            .to_string();
        if url.contains("\" title=") {
            url = url.replace("\" title=\"", "").trim().to_string();
        }
        let title = cap[3].to_string().trim().to_string();
        let date = cap[4].to_string();
        // let mut pending_url =
        //     fetch_chapter_page(&url, &title, &date, &comic_id, &client, proxy.clone()).await;
        // let mut tries = 0;
        // while pending_url.is_none() {
        //     let wait_time = rand::thread_rng().gen_range(1..5);
        //     tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
        //     pending_url =
        //         fetch_chapter_page(&url, &title, &date, &comic_id, &client, proxy.clone()).await;
        //     // println!("retry {}", url);
        //     if tries > 10 {
        //         break;
        //     }
        //     tries += 1;
        // }
        // // combine pending url with result
        // result.extend(pending_url.unwrap().clone());
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
            if tmp.unwrap().is_some() {
                println!("chapter {} already exists", url);
                result.push(url.to_string());
                continue;
            }
        }
        // let chapter = client
        //     .chapter()
        //     .create(
        //         title.to_string(),
        //         url.to_string(),
        //         prisma::comic::UniqueWhereParam::IdEquals(comic_id.to_string()),
        //         date.to_string(),
        //         vec![],
        //     )
        //     .exec()
        //     .await;
        // if chapter.is_err() {
        //     continue;
        // }
        chapters.push(prisma::chapter::create_unchecked(
            title.to_string(),
            url.to_string(),
            comic_id.to_string(),
            date.to_string(),
            vec![],
        ));
        result.push(url.to_string());
    }
    client.chapter().create_many(chapters).exec().await.unwrap();
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
        if tmp.unwrap().is_none() {
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
        } else {
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
    // TODO: enable this sleep code if rate limit is fixed
    // let wait_time = rand::thread_rng().gen_range(1..3);
    // tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
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
    // let re = Regex::new(r#"href="([^"]+)"#).unwrap();
    // for cap in re.captures_iter(&html) {
    //     // dbg!(&cap[1]);
    //     let url = process_url(&cap[1]);
    //     if url.is_some() {
    //         result.push(url.unwrap());
    //     }
    // }
    Some(result)
}
