use std::sync::Arc;

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
    // println!("fetching comic page {}", url);
    let mut result = Vec::new();
    // fetch all urls from page

    Some(result)
}

pub fn process_url(url: &str) -> Option<String> {
    if url.starts_with("//id.blogtruyenmoi.com") {
        return Some(format!("https:{}", url).trim().to_string());
    }
    if url.starts_with("/") {
        return Some(
            format!("https://blogtruyenmoi.com{}", url)
                .trim()
                .to_string(),
        );
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
            ThreadMessage::Start(url) => {
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
                        tx.send(ThreadMessage::Retry(url.clone())).await.unwrap();
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
                            vec![prisma::urls::fetching::set(true)],
                        )
                        .exec()
                        .await;
                    if tmp.is_err() {
                        tx.send(ThreadMessage::Retry(url.clone())).await.unwrap();
                        continue;
                    }
                };
                println!("worker {} fetching {}", worker_id, url);
                let http_client = reqwest::Client::new();
                let resp = http_client
                    .get(url.clone())
                    .header("User-Agent", "Mozilla/5.0")
                    .header("Referrer", "https://blogtruyenmoi.com/")
                    .send()
                    .await;
                if resp.is_err() {
                    tx.send(ThreadMessage::Retry(url.clone())).await.unwrap();
                    continue;
                }
                let html = {
                    let tmp = resp.unwrap().text().await;
                    if tmp.is_err() {
                        tx.send(ThreadMessage::Retry(url.clone())).await.unwrap();
                    }
                    tmp.unwrap()
                };
                // println!("worker {} fetched {}", worker_id, url);
                let mut result = Vec::new();
                let re = Regex::new(r#"href="([^"]+)"#).unwrap();
                for cap in re.captures_iter(&html) {
                    // dbg!(&cap[1]);
                    let url = process_url(&cap[1]);
                    if url.is_some() {
                        result.push(url.unwrap());
                    }
                }
                if is_comic_page(&html) {
                    // only chapter pending url because we had fetch comic page pending url before
                    let pending_url_comic = {
                        let tmp = parse_comic_page(&html, &url, client.clone()).await;
                        if tmp.is_none() {
                            tx.send(ThreadMessage::Retry(url.clone())).await.unwrap();
                        }
                        tmp.unwrap()
                    };
                    result.extend(pending_url_comic);
                }
                {
                    let tmp = client
                        .lock()
                        .await
                        .urls()
                        .update_many(
                            vec![prisma::urls::url::equals(url.clone())],
                            vec![prisma::urls::fetched::set(true)],
                        )
                        .exec()
                        .await;
                    if tmp.is_err() {
                        tx.send(ThreadMessage::Retry(url.clone())).await.unwrap();
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
