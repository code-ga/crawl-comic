use std::sync::Arc;

use regex::Regex;
use tokio::sync::Mutex;

use crate::{
    db::{DbUtils, UpdateUrlDocFields},
    types::thread_message::ThreadMessage,
    util::{self, get_host},
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

mod mangajuice;
use mangajuice::{
    is_chapter_page as is_mangajuice_chapter_page, is_comic_page as is_mangajuice_comic_page,
    parse_chapter_page as parse_mangajuice_chapter_page,
    parse_comic_page as parse_mangajuice_comic_page,
};

mod alyasometimeshidesherfeelings;
use alyasometimeshidesherfeelings::{
    is_chapter_page as is_alyasometimeshidesherfeelings_chapter_page,
    is_comic_page as is_alyasometimeshidesherfeelings_comic_page,
    parse_chapter_page as parse_alyasometimeshidesherfeelings_chapter_page,
    parse_comic_page as parse_alyasometimeshidesherfeelings_comic_page,
};

pub static ACCEPTED_HOSTS: [&str; 13] = [
    "blogtruyenmoi.com",
    "nettruyenee.com",
    "www.nettruyenee.com",
    "www.nettruyenff.com",
    "nettruyenff.com",
    "nettruyenbb.com",
    "www.nettruyenbb.com",
    "www.nettruyenvv.com",
    "nettruyenvv.com",
    "nettruyentt.com",
    "www.nettruyentt.com",
    "mangajuice.com",
    "alyasometimeshidesherfeelings.com",
];
pub static NETTRUYEN_HOSTS: [&str; 10] = [
    "nettruyenee.com",
    "www.nettruyenee.com",
    "nettruyenff.com",
    "www.nettruyenff.com",
    "nettruyenbb.com",
    "www.nettruyenbb.com",
    "www.nettruyenvv.com",
    "nettruyenvv.com",
    "nettruyentt.com",
    "www.nettruyentt.com",
];

pub fn process_url(url: &str, now_url: &str) -> Option<String> {
    // if !url.starts_with("https://") && !url.starts_with("http://") {
    //     return None;
    // }
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
    } else if host.contains("nettruyen") {
        let url_host = get_host(&url);
        if url_host.is_none() {
            return None;
        }
        let url_host = url_host.unwrap();
        log::debug!("url_host: {}", url_host);
        if !NETTRUYEN_HOSTS.contains(&url_host.as_str()) {
            return None;
        }
        let url = url.trim().to_string().replace("www.", "");
        if url.ends_with("#nt_listchapter") {
            return None;
        }
        if url.ends_with("#nt_comment") {
            return None;
        }
        if url.starts_with(format!("https://{}/truyen-tranh/", host).as_str()) {
            return Some(url);
        }
        if url.starts_with(format!("https://{}/tim-truyen?page=", host).as_str()) {
            return Some(url);
        }
        if is_nettruyenee_chapter_page(&url, "") {
            return Some(url);
        }
    } else if host.contains("mangajuice.com") {
        let url = url.trim().to_string();
        if url.starts_with("https://mangajuice.com/manga")
            && !url.starts_with("https://mangajuice.com/manga-status/")
        {
            return Some(url);
        }
        if url.starts_with("https://mangajuice.com/chapter") && !url.ends_with("/feed/") {
            return Some(url);
        }
        if url.starts_with("https://mangajuice.com/updates") {
            return Some(url);
        }
        if url.starts_with("https://mangajuice.com/newest") {
            return Some(url);
        }
    } else if host.contains("alyasometimeshidesherfeelings.com") {
        // by @thedtvn request
        let url = url.trim().to_string();
        if url.ends_with("/feed/") {
            return None;
        }
        if url.eq("https://alyasometimeshidesherfeelings.com/") {
            return Some(url);
        }
        if url.starts_with("https://alyasometimeshidesherfeelings.com/manga/") {
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
    let client = {
        let tmp = DbUtils::new().await;
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
                if !ACCEPTED_HOSTS.contains(&hostname.as_str())
                    && !NETTRUYEN_HOSTS.contains(&hostname.as_str())
                {
                    log::info!("{} is not accepted host", hostname);
                    {
                        let tmp = client
                            .lock()
                            .await
                            .update_url_doc(
                                url.clone(),
                                vec![
                                    UpdateUrlDocFields::Fetched(true),
                                    UpdateUrlDocFields::Fetching(false),
                                ],
                            )
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

                if NETTRUYEN_HOSTS.contains(&hostname.as_str()) {
                    log::info!("{} in time error show we skip that", hostname);
                    {
                        let tmp = client
                            .lock()
                            .await
                            .update_url_doc(
                                url.clone(),
                                vec![
                                    UpdateUrlDocFields::Fetched(true),
                                    UpdateUrlDocFields::Fetching(false),
                                ],
                            )
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

                log::info!("worker {} fetching {}", worker_id, url);
                let html = {
                    let client = client.lock().await;
                    match util::fetch_page::fetch_page(&client, &hostname, worker_id, url.clone())
                        .await
                    {
                        Ok(html) => {
                            if html.len() > 0 {
                                html
                            } else {
                                tx.send(ThreadMessage::Done(vec![], url, false))
                                    .await
                                    .unwrap();
                                continue;
                            }
                        }
                        Err(e) => {
                            log::info!("worker {} failed {} error {:#?}", worker_id, url, e);
                            tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                .await
                                .unwrap();
                            continue;
                        }
                    }
                };

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
                            result.extend(tmp.unwrap());
                        }
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
                } else if hostname.contains("nettruyen") {
                    if is_nettruyenee_chapter_page(&url, &html) {
                        let client = client.lock().await;
                        let pending_url_chapter = {
                            let tmp = parse_nettruyenee_chapter_page(&url, &html, &client).await;
                            if tmp.is_none() {
                                log::info!("failed to parse chapter page {}", url);
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
                                log::info!("failed to parse comic page {}", url);
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                                continue;
                            }
                            tmp.unwrap()
                        };
                        result.extend(pending_url_comic.clone());
                    }
                } else if hostname.contains("mangajuice.com") {
                    if is_mangajuice_chapter_page(&url, &html) {
                        let client = client.lock().await;
                        let pending_url_chapter = {
                            let tmp = parse_mangajuice_chapter_page(&url, &html, &client).await;
                            if tmp.is_none() {
                                log::info!("failed to parse chapter page {}", url);
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                                continue;
                            }
                            tmp.unwrap()
                        };
                        result.extend(pending_url_chapter.clone());
                    } else if is_mangajuice_comic_page(&url, &html) {
                        let pending_url_comic = {
                            let tmp =
                                parse_mangajuice_comic_page(&html, &url, client.clone()).await;
                            if tmp.is_none() {
                                log::info!("failed to parse comic page {}", url);
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                                continue;
                            }
                            tmp.unwrap()
                        };
                        result.extend(pending_url_comic.clone());
                    }
                } else if hostname.contains("alyasometimeshidesherfeelings.com") {
                    if is_alyasometimeshidesherfeelings_chapter_page(&url, &html) {
                        let client = client.lock().await;
                        let pending_url_chapter = {
                            let tmp = parse_alyasometimeshidesherfeelings_chapter_page(
                                &url, &html, &client,
                            )
                            .await;
                            if tmp.is_none() {
                                log::info!("failed to parse chapter page {}", url);
                                tx.send(ThreadMessage::Retry(url.clone(), i_tries))
                                    .await
                                    .unwrap();
                                continue;
                            }
                            tmp.unwrap()
                        };
                        result.extend(pending_url_chapter.clone());
                    }
                    if is_alyasometimeshidesherfeelings_comic_page(&url, &html) {
                        let pending_url_comic = {
                            let tmp = parse_alyasometimeshidesherfeelings_comic_page(
                                &html,
                                &url,
                                client.clone(),
                            )
                            .await;
                            if tmp.is_none() {
                                log::info!("failed to parse comic page {}", url);
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
                {
                    let tmp = client
                        .lock()
                        .await
                        .update_url_doc(
                            url.clone(),
                            vec![
                                UpdateUrlDocFields::Fetched(true),
                                UpdateUrlDocFields::Fetching(false),
                            ],
                        )
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
                result = result
                    .into_iter()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                tx.send(ThreadMessage::Done(result, url, false))
                    .await
                    .unwrap();
            }
            ThreadMessage::Exited(i) => {
                if i == worker_id {
                    return ();
                }
            }
            _ => {}
        }
    }
}
