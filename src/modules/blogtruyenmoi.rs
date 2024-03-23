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
            .find_unique(prisma::comic::UniqueWhereParam::UrlEquals(url.to_string()))
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
    let (_, comic_info) = parse_blog_truyen_moi_html_page(&page.to_string());
    // append comic info to update field
    for s in comic_info {
        update_field.push(s.clone());
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
                .find_unique(prisma::chapter::UniqueWhereParam::UrlEquals(
                    url.to_string(),
                ))
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

    if !comic_exec.python_fetch_info {
        let tmp = client
            .html()
            .find_unique(prisma::html::UniqueWhereParam::UrlEquals(url.to_string()))
            .exec()
            .await;
        if tmp.is_err() {
            return None;
        }
        let tmp = tmp.unwrap();
        if tmp.is_none() {
            if let Err(_) = client
                .html()
                .create(url.to_string(), page.to_string(), vec![])
                .exec()
                .await
            {
                return None;
            }
        }
    }

    Some(result)
}

pub async fn parse_chapter_page(
    url: &str,
    html: &str,
    client: &PrismaClient,
) -> Option<Vec<String>> {
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
    Some(result)
}

use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;

pub fn parse_blog_truyen_moi_html_page(
    html: &str,
) -> (
    HashMap<String, serde_json::Value>,
    Vec<prisma::comic::SetParam>,
) {
    let mut result = HashMap::new();
    let mut update_data = Vec::new();
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
        update_data.push(prisma::comic::content::set(Some(content.to_string())));
        result.insert("content".to_string(), json!(content.to_string()));
    }
    let thumbnail_selector = Selector::parse(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.thumbnail > img",
    )
    .unwrap();
    let thumbnail = document.select(&thumbnail_selector).next();
    if let Some(thumbnail) = thumbnail {
        let thumbnail = thumbnail.value().attr("src").unwrap().trim().to_string();
        update_data.push(prisma::comic::thumbnail::set(Some(thumbnail.to_string())));
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
                update_data.push(prisma::comic::another_name::set(another_name.clone()));
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
                update_data.push(prisma::comic::author::set(json!(author.clone())));
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
                update_data.push(prisma::comic::source::set(json!(source.clone())));
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
                update_data.push(prisma::comic::translator_team::set(json!(
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
                update_data.push(prisma::comic::posted_by::set(json!(posted_by.clone())));
                result.insert("postedBy".to_string(), json!(posted_by));
                let status_selector = Selector::parse("span").unwrap();
                let status = p.select(&status_selector).next();
                if let Some(status) = status {
                    let status = status.text().collect::<String>();
                    update_data.push(prisma::comic::status::set(status.trim().to_string()));
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
                update_data.push(prisma::comic::genre::set(json!(genre.clone())));
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
    async fn test_parse_blog_truyen_moi_html_page() {
        let client = reqwest::Client::new();
        let html = client
            .get("https://blogtruyenmoi.com/34464/kiyota-san-muon-bi-vay-ban")
            .header("User-Agent", "Mozilla/5.0")
            .header("Referrer", "https://blogtruyenmoi.com/")
            .send()
            .await
            .unwrap();
        let html = html.text().await.unwrap();
        let (result, _) = super::parse_blog_truyen_moi_html_page(&html);
        // print out result
        println!("{:?}", json!(result));
    }
}
