use std::sync::Arc;

use regex::Regex;
use tokio::sync::Mutex;

use crate::{
    prisma::{self, PrismaClient},
    types::thread_message::ThreadMessage,
    util::get_host,
};
use rand::Rng;

mod blogtruyenmoi;
use blogtruyenmoi::{
    is_comic_page as is_blogtruyenmoi_comic_page,
    parse_chapter_page as parse_blogtruyenmoi_chapter_page,
    parse_comic_page as parse_blogtruyenmoi_comic_page,
};
mod nettruyenee;
use nettruyenee::{
    is_chapter_page as is_nettruyenee_chapter_page, is_comic_page as is_nettruyenee_comic_page,
    parse_chapter_page as parse_nettruyenee_chapter_page,
    parse_comic_page as parse_nettruyenee_comic_page,
};

pub static ACCEPTED_HOSTS: [&str; 3] = [
    "blogtruyenmoi.com",
    "nettruyenee.com",
    "www.nettruyenee.com",
];

pub fn process_url(url: &str, now_url: &str) -> Option<String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return None;
    }
    let host = get_host(now_url);
    if host.is_none() {
        return None;
    }
    let host = host.unwrap();
    if !ACCEPTED_HOSTS.contains(&host.as_str()) {
        return None;
    }
    if host.contains("blogtruyenmoi.com") {
        let url = url.trim().to_string();
        if url.starts_with("//id.blogtruyenmoi.com") {
            // return Some(format!("https:{}", url).trim().to_string());
            return None;
        }
        if url.ends_with("#bt-comment") {
            return None;
        }
        if url.starts_with("/c") {
            return Some(format!("https://blogtruyenmoi.com{}", url));
        }
        if url.starts_with("javascript:LoadListMangaPage(") {
            // javascript:LoadListMangaPage(2) -> https://blogtruyenmoi.com/ajax/Search/AjaxLoadListManga?key=tatca&orderBy=3&p=2
            let re = Regex::new(r#"LoadListMangaPage\((\d+)\)"#).unwrap();
            let cap = re.captures(&url).unwrap();
            let page = cap[1].to_string();
            return Some(format!(
                "https://blogtruyenmoi.com/ajax/Search/AjaxLoadListManga?key=tatca&orderBy=3&p={}",
                page
            ));
        }
        if url.starts_with("/") {
            let comic_regex = Regex::new(r#"/\d+/.+"#).unwrap();
            if comic_regex.is_match(&url) {
                return Some(
                    format!("https://blogtruyenmoi.com{}", url)
                        .trim()
                        .to_string(),
                );
            }
            return None;
        }
    } else if host.contains("nettruyenee.com") {
        let url = url.trim().to_string();
        if url.ends_with("#nt_listchapter") {
            return None;
        }
        if url.starts_with("https://www.nettruyenee.com/truyen-tranh/") {
            return Some(url);
        }
        if url.starts_with("https://www.nettruyenee.com/tim-truyen?page=") {
            return Some(url);
        }
        if is_nettruyenee_chapter_page(&url, "") {
            return Some(url);
        }
    }

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
                let hostname = get_host(&url).unwrap();
                if !ACCEPTED_HOSTS.contains(&hostname.as_str()) {
                    {
                        let tmp = client
                            .lock()
                            .await
                            .urls()
                            .update(
                                prisma::urls::UniqueWhereParam::UrlEquals(url.clone()),
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
                    tx.send(ThreadMessage::Done(vec![], url, false))
                        .await
                        .unwrap();
                    continue;
                }

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

                let (http_client, _) = {
                    let client = client.lock().await;
                    let proxy = crate::util::get_proxy(&client).await;
                    let mut http_client = reqwest::ClientBuilder::new();
                    if proxy.is_some() {
                        let proxy = proxy.clone().unwrap();
                        println!("using proxy {} - {}", proxy.id, proxy.url);
                        http_client = http_client.proxy({
                            let client_proxy =
                                reqwest::Proxy::all(proxy.url.to_string().trim().to_string());
                            if client_proxy.is_err() {
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                                continue;
                            }
                            let mut client_proxy = client_proxy.unwrap();
                            if proxy.username.is_some() && proxy.password.is_some() {
                                client_proxy = client_proxy
                                    .basic_auth(&proxy.username.unwrap(), &proxy.password.unwrap())
                            }
                            client_proxy
                        });
                    }
                    (http_client.build().unwrap(), proxy)
                };

                println!("worker {} fetching {}", worker_id, url);

                let mut rep = http_client
                    .get(url.clone())
                    .header("User-Agent", "Mozilla/5.0");
                if hostname.contains("blogtruyenmoi.com") {
                    rep = rep.header("Referrer", format!("https://{}/", hostname.to_string()));
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

                // println!("worker {} fetched {}", worker_id, url);
                let mut result = Vec::new();
                let re = Regex::new(r#"href=["|']([^"']+)"#).unwrap();
                for cap in re.captures_iter(&html) {
                    // dbg!(&cap[1]);
                    let url = process_url(&cap[1], &url);
                    if url.is_some() {
                        result.push(url.unwrap());
                    }
                }

                if hostname.contains("blogtruyenmoi.com") {
                    // chapter page
                    if url.starts_with("https://blogtruyenmoi.com/c") {
                        // fetch chap
                        let client = client.lock().await;
                        let tmp = parse_blogtruyenmoi_chapter_page(&url, &html, &client).await;
                        if tmp.is_none() {
                            {
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                            }
                            continue;
                        }
                        if tmp.is_some() {
                            // tx.send(ThreadMessage::Done(tmp.unwrap(), url.clone(), false))
                            //     .await
                            //     .unwrap();
                            result.extend(tmp.unwrap());
                        }
                        // {
                        //     let tmp = client
                        //         .urls()
                        //         .update_many(
                        //             vec![prisma::urls::url::equals(url.clone())],
                        //             vec![
                        //                 prisma::urls::fetched::set(true),
                        //                 prisma::urls::fetching::set(false),
                        //             ],
                        //         )
                        //         .exec()
                        //         .await;
                        //     if tmp.is_err() {
                        //         {
                        //             tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                        //                 .await
                        //                 .unwrap();
                        //         }
                        //         continue;
                        //     }
                        // }
                    } else if is_blogtruyenmoi_comic_page(&html) {
                        // only chapter pending url because we had fetch comic page pending url before
                        let pending_url_comic = {
                            let tmp =
                                parse_blogtruyenmoi_comic_page(&html, &url, client.clone()).await;
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
                } else if hostname.contains("nettruyenee.com") {
                    if is_nettruyenee_chapter_page(&url, &html) {
                        let client = client.lock().await;
                        let pending_url_chapter = {
                            let tmp = parse_nettruyenee_chapter_page(&url, &html, &client).await;
                            if tmp.is_none() {
                                println!("failed to parse chapter page {}", url);
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                                continue;
                            }
                            tmp.unwrap()
                        };
                        result.extend(pending_url_chapter.clone());
                    } else if is_nettruyenee_comic_page(&url, &html) {
                        let pending_url_comic = {
                            let tmp =
                                parse_nettruyenee_comic_page(&html, &url, client.clone()).await;
                            if tmp.is_none() {
                                println!("failed to parse comic page {}", url);
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                                continue;
                            }
                            tmp.unwrap()
                        };
                        result.extend(pending_url_comic.clone());
                    }
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
                        .update(
                            prisma::urls::UniqueWhereParam::UrlEquals(url.clone()),
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
