use std::sync::Arc;

use rand::Rng;
use regex::Regex;
use tokio::sync::Mutex;
// use lazy_static::lazy_static;

use crate::{
    prisma::{self, PrismaClient},
    types::thread_message::ThreadMessage,
};

pub fn is_comic_page(page: &str) -> bool {
    page.contains("Thêm vào bookmark")
}

pub async fn parse_comic_page(
    page: &str,
    url: &str,
    client: Arc<Mutex<PrismaClient>>,
) -> Option<Vec<String>> {
    let client = client.lock().await;
    {
        let tmp = client
            .comic()
            .find_first(vec![prisma::comic::url::equals(url.to_string())])
            .exec()
            .await;
        if tmp.is_err() {
            return None;
        }
        if tmp.unwrap().is_some() {
            return Some(Vec::new());
        }
    }
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

async fn fetch_chapter_page(
    url: &str,
    client: &PrismaClient,
    proxy: Option<prisma::proxy::Data>,
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
    let mut result = Vec::new();

    let mut http_client = reqwest::ClientBuilder::new();
    if proxy.is_some() {
        let proxy = proxy.clone().unwrap();
        http_client = http_client
            .proxy(reqwest::Proxy::all(proxy.url.to_string().trim().to_string()).unwrap());
    }
    let http_client = http_client.build().unwrap();
    let req = http_client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Referrer", "https://blogtruyenmoi.com/");
    if proxy.is_some() {
        // proxy auth
        let _proxy = proxy.clone().unwrap();
        // req = req.header(
        //     "Proxy-Authorization",
        //     format!("{}:{}", proxy.username, proxy.password),
        // );
    }
    let resp = req.send().await;
    if resp.is_err() {
        return None;
    }
    let resp = resp.unwrap();
    if resp.status().is_success() == false && resp.status().as_u16().eq(&404) == false {
        println!("fetch chapter failed {} status {}", url, resp.status());
        if resp.status().as_u16().eq(&404) {
            return Some(vec![]);
        }
        if resp.status().as_u16().eq(&429) {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
        return None;
    }
    // if resp.status().as_u16().eq(&404) {
    //     return Some(vec![]);
    // }

    let html = {
        let tmp = resp.text().await;
        if tmp.is_err() {
            return None;
        }
        tmp.unwrap()
    };

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
    let re = Regex::new(r#"href="([^"]+)"#).unwrap();
    for cap in re.captures_iter(&html) {
        // dbg!(&cap[1]);
        let url = process_url(&cap[1]);
        if url.is_some() {
            result.push(url.unwrap());
        }
    }
    Some(result)
}

pub fn process_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.starts_with("//id.blogtruyenmoi.com") {
        // return Some(format!("https:{}", url).trim().to_string());
        return None;
    }
    if url.starts_with("/c") {
        return Some(format!("https://blogtruyenmoi.com{}", url));
    }
    if url.starts_with("javascript:LoadListMangaPage(") {
        // javascript:LoadListMangaPage(2) -> https://blogtruyenmoi.com/ajax/Search/AjaxLoadListManga?key=tatca&orderBy=3&p=2
        let re = Regex::new(r#"LoadListMangaPage\((\d+)\)"#).unwrap();
        let cap = re.captures(url).unwrap();
        let page = cap[1].to_string();
        return Some(format!(
            "https://blogtruyenmoi.com/ajax/Search/AjaxLoadListManga?key=tatca&orderBy=3&p={}",
            page
        ));
    }
    if url.starts_with("/") {
        let comic_regex = Regex::new(r#"/\d+/.+"#).unwrap();
        if comic_regex.is_match(url) {
            return Some(
                format!("https://blogtruyenmoi.com{}", url)
                    .trim()
                    .to_string(),
            );
        }
        return None;
    }
    // if url.starts_with("https://vlogtruyen11.net") {
    //     return Some(url.to_string());
    // }
    // if url.starts_with("/") {
    //     return Some(format!("https://vlogtruyen11.net{}", url));
    // }
    None
}

pub async fn thread_worker(
    tx: async_channel::Sender<ThreadMessage>,
    rx: async_channel::Receiver<ThreadMessage>,
    worker_id: usize,
) {
    let client: PrismaClient = {
        let tmp = PrismaClient::_builder().build().await;
        if tmp.is_err() {
            tx.send(ThreadMessage::Stop(worker_id)).await.unwrap();
        }
        tmp.unwrap()
    };

    let client = Arc::new(Mutex::new(client));

    loop {
        let job = rx.recv().await.unwrap();
        match job {
            ThreadMessage::Start(url, i_tries) => {
                let wait_time = rand::thread_rng().gen_range(1..10);
                tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                {
                    let tmp = client
                        .lock()
                        .await
                        .urls()
                        .find_first(vec![
                            prisma::urls::url::equals(url.clone()),
                            prisma_client_rust::operator::or(vec![
                                prisma::urls::fetched::equals(true),
                                prisma::urls::fetching::equals(false),
                            ]),
                        ])
                        .exec()
                        .await;
                    if tmp.is_err() {
                        {
                            tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                .await
                                .unwrap();
                        }
                        continue;
                    }
                    if tmp.unwrap().is_some() {
                        // already fetched emit done
                        tx.send(ThreadMessage::Done(vec![], url.clone(), true))
                            .await
                            .unwrap();
                        println!("worker {} already fetched {}", worker_id, url);
                        continue;
                    }
                };
                // TODO: will remove to performance
                {
                    // update fetched
                    let tmp = client
                        .lock()
                        .await
                        .urls()
                        .update_many(
                            vec![prisma::urls::url::equals(url.clone())],
                            vec![
                                prisma::urls::fetching::set(true),
                                prisma::urls::fetched::set(false),
                            ],
                        )
                        .exec()
                        .await;
                    if tmp.is_err() {
                        {
                            tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                .await
                                .unwrap();
                        }
                        continue;
                    }
                };

                let (http_client, proxy) = {
                    let client = client.lock().await;
                    let proxy = crate::util::get_proxy(&client).await;
                    let mut http_client = reqwest::ClientBuilder::new();
                    if proxy.is_some() {
                        let proxy = proxy.clone().unwrap();
                        println!("using proxy {}", proxy.url);
                        http_client = http_client.proxy(
                            reqwest::Proxy::all(proxy.url.to_string().trim().to_string()).unwrap(),
                        );
                    }
                    (http_client.build().unwrap(), proxy)
                };
                if url.starts_with("https://blogtruyenmoi.com/c") {
                    // fetch chap
                    let client = client.lock().await;
                    let tmp = fetch_chapter_page(&url, &client, proxy.clone()).await;
                    if tmp.is_none() {
                        {
                            tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                .await
                                .unwrap();
                        }
                        continue;
                    }
                    if tmp.is_some() {
                        tx.send(ThreadMessage::Done(tmp.unwrap(), url.clone(), false))
                            .await
                            .unwrap();
                    }
                    {
                        let tmp = client
                            .urls()
                            .update_many(
                                vec![prisma::urls::url::equals(url.clone())],
                                vec![
                                    prisma::urls::fetched::set(true),
                                    prisma::urls::fetching::set(false),
                                ],
                            )
                            .exec()
                            .await;
                        if tmp.is_err() {
                            {
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                            }
                            continue;
                        }
                    }
                    continue;
                }
                println!("worker {} fetching {}", worker_id, url);

                let rep = http_client
                    .get(url.clone())
                    .header("User-Agent", "Mozilla/5.0")
                    .header("Referrer", "https://blogtruyenmoi.com/");
                if proxy.is_some() {
                    let _proxy = proxy.clone().unwrap();
                    // dbg!(&proxy);
                    // rep = rep.header("Proxy-Authorization", proxy.auth);
                }
                let resp = rep.send().await;
                if resp.is_err() {
                    {
                        tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                            .await
                            .unwrap();
                    }
                    continue;
                }
                if resp.as_ref().unwrap().status().is_success() == false
                    && resp.as_ref().unwrap().status().as_u16().eq(&404) == false
                {
                    if resp.as_ref().unwrap().status().as_u16().eq(&429) {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                    {
                        tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                            .await
                            .unwrap();
                    }
                    println!(
                        "worker {} failed {} status : {}",
                        worker_id,
                        url,
                        resp.unwrap().status()
                    );
                    continue;
                }
                let html = {
                    let tmp = resp.unwrap().text().await;
                    if tmp.is_err() {
                        {
                            tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                .await
                                .unwrap();
                        }
                        continue;
                    }
                    tmp.unwrap()
                };
                // dbg!(&html);
                // println!("worker {} fetched {}", worker_id, url);
                let mut result = Vec::new();
                let re = Regex::new(r#"href=["|']([^"']+)"#).unwrap();
                for cap in re.captures_iter(&html) {
                    // dbg!(&cap[1]);
                    let url = process_url(&cap[1]);
                    if url.is_some() {
                        result.push(url.unwrap());
                    }
                }
                if is_comic_page(&html) {
                    let wait_time = rand::thread_rng().gen_range(1..5);
                    tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                    // only chapter pending url because we had fetch comic page pending url before
                    let pending_url_comic = {
                        let tmp = parse_comic_page(&html, &url, client.clone()).await;
                        if tmp.is_none() {
                            {
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                            }
                            continue;
                        }
                        tmp.unwrap()
                    };
                    result.extend(pending_url_comic.clone());
                }
                result = result
                    .into_iter()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                {
                    let tmp = client
                        .lock()
                        .await
                        .urls()
                        .update_many(
                            vec![prisma::urls::url::equals(url.clone())],
                            vec![
                                prisma::urls::fetched::set(true),
                                prisma::urls::fetching::set(false),
                            ],
                        )
                        .exec()
                        .await;
                    if tmp.is_err() {
                        {
                            tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                .await
                                .unwrap();
                        }
                        continue;
                    }
                }
                tx.send(ThreadMessage::Done(result, url, false))
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}
