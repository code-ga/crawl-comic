use std::{
    panic, process,
    sync::{Arc, Mutex},
};

use rand::Rng;

use types::thread_message::ThreadMessage;

pub mod db;
mod modules;
pub mod prisma;
mod types;
mod util;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let client = db::DbUtils::new().await.unwrap();

    let num_of_threads = 10;
    let init_url = "https://blogtruyenmoi.com/33534/moi-tinh-giua-ten-lua-dao-va-nu-canh-sat".to_string();
    {
        // let tmp = client
        //     .urls()
        //     .find_unique(prisma::urls::UniqueWhereParam::UrlEquals(init_url.clone()))
        //     .exec()
        //     .await
        //     .unwrap();
        let tmp = client.find_url_doc_by_url(&init_url.clone()).await.unwrap();
        if tmp.is_none() {
            // client
            //     .urls()
            //     .create(init_url.clone(), vec![])
            //     .exec()
            //     .await
            //     .unwrap();
            client.create_url_doc(&init_url.clone()).await.unwrap();
        }
    };
    // worker to main channel
    let (main_tx, main_rx) = async_channel::bounded::<ThreadMessage>(num_of_threads);
    // main to worker channel
    let (worker_tx, worker_rx) = async_channel::bounded::<ThreadMessage>(num_of_threads);
    // let rx = Arc::new(Mutex::new(worker_rx));

    let fetching_url = Arc::new(Mutex::new(std::collections::HashSet::new()));
    fetching_url
        .clone()
        .lock()
        .unwrap()
        .insert(init_url.clone());

    {
        let fetching_url = fetching_url.clone();
        let orig_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // invoke the default handler and exit the process
            orig_hook(panic_info);
            let _ = async {
                let client = db::DbUtils::new().await.unwrap();
                // update fetched = false for fetching url
                let mut update_data = vec![];
                for url in &fetching_url.clone().lock().unwrap().clone() {
                    // update_data.push(client.urls().update(
                    //     prisma::urls::UniqueWhereParam::UrlEquals(url.clone()),
                    //     vec![prisma::urls::fetched::set(false)],
                    // ));
                    update_data.push(client.update_url_doc_daft(
                        url.to_string(),
                        vec![db::UpdateUrlDocFields::Fetched(false)],
                    ));
                }
                let _ = client._batch(update_data).await;
            };
            process::exit(1);
        }));
    }

    let mut workers = Vec::new();
    for i in 0..num_of_threads {
        log::info!("spawn {}", i);
        let tx = main_tx.clone();
        let rx = worker_rx.clone();
        let worker = tokio::spawn(async move {
            modules::thread_worker(tx, rx, i).await;
        });

        workers.push(types::Worker {
            id: i,
            join_handle: worker,
        });
    }

    worker_tx
        .send(types::thread_message::ThreadMessage::Start(init_url, 0))
        .await
        .unwrap();

    let fetching_url = fetching_url.clone();

    let term = Arc::new(std::sync::atomic::AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();
    while !term.load(std::sync::atomic::Ordering::Relaxed) {
        if worker_rx.is_empty() {
            let wait_time = rand::thread_rng().gen_range(1..5);
            tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
            let mut pending_url = client
                .get_pending_urls(num_of_threads - worker_rx.len(), "".to_string())
                .await;
            if !pending_url.is_empty() {
                while !worker_rx.is_full() {
                    let pending_url = {
                        let tmp = pending_url.pop();
                        if tmp.is_none() {
                            break;
                        }
                        tmp.unwrap()
                    };

                    fetching_url.lock().unwrap().insert(pending_url.to_string());
                    worker_tx
                        .send(types::thread_message::ThreadMessage::Start(
                            pending_url.to_string(),
                            0,
                        ))
                        .await
                        .unwrap();
                }
            }
        }
        let job = main_rx.recv().await.unwrap();
        match job {
            ThreadMessage::Stop(id) => {
                // spawn new worker and replace old
                let tx = main_tx.clone();
                let rx = worker_rx.clone();
                let worker = tokio::spawn(async move {
                    modules::thread_worker(tx, rx, id).await;
                });
                workers[id].join_handle = worker;
            }
            ThreadMessage::Retry(url, i) => {
                let wait_time = rand::thread_rng().gen_range(1..5);
                tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                if i >= 10 {
                    // client
                    //     .urls()
                    //     .update(
                    //         prisma::urls::UniqueWhereParam::UrlEquals(url.to_string()),
                    //         vec![
                    //             prisma::urls::is_error::set(true),
                    //             prisma::urls::fetched::set(true),
                    //         ],
                    //     )
                    //     .exec()
                    //     .await
                    //     .unwrap();
                    client
                        .update_url_doc(
                            url.to_string(),
                            vec![
                                db::UpdateUrlDocFields::Fetched(true),
                                db::UpdateUrlDocFields::IsError(true),
                            ],
                        )
                        .await
                        .unwrap();
                    fetching_url.lock().unwrap().remove(&url);
                    let mut pending_urls = client
                        .get_pending_urls(num_of_threads - worker_rx.len(), url.clone())
                        .await;
                    // dbg!(&pending_urls);
                    while !worker_rx.is_full() {
                        let pending_url = {
                            let tmp = pending_urls.pop();
                            if tmp.is_none() {
                                let wait_time = rand::thread_rng().gen_range(1..5);
                                tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                                pending_urls = client
                                    .get_pending_urls(num_of_threads - worker_rx.len(), url.clone())
                                    .await;
                                continue;
                                // break;
                            }
                            tmp.unwrap()
                        };
                        fetching_url.lock().unwrap().insert(pending_url.to_string());
                        // let wait_time = rand::thread_rng().gen_range(1..5);
                        // tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                        worker_tx
                            .send(types::thread_message::ThreadMessage::Start(
                                pending_url.to_string(),
                                0,
                            ))
                            .await
                            .unwrap();
                    }
                    continue;
                }
                log::info!("retry {}", url);
                worker_tx
                    .send(types::thread_message::ThreadMessage::Start(url, i + 1))
                    .await
                    .unwrap();
            }
            ThreadMessage::Done(worker_pending_url, comic_url, _) => {
                for u in worker_pending_url {
                    let url = client.filters_urls(u).await;
                    if url.is_none() {
                        continue;
                    };
                    // client
                    //     .urls()
                    //     .create(url.unwrap().to_string(), vec![])
                    //     .exec()
                    //     .await
                    //     .unwrap();
                    client
                        .create_url_doc(&url.unwrap().to_string())
                        .await
                        .unwrap();
                }

                fetching_url.lock().unwrap().remove(&comic_url);

                let wait_time = rand::thread_rng().gen_range(1..5);
                tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                // sleep 1s
                let mut pending_urls = client
                    .get_pending_urls(num_of_threads - worker_rx.len(), comic_url.clone())
                    .await;
                // dbg!(&pending_urls);
                while !worker_rx.is_full() {
                    let pending_url = {
                        let tmp = pending_urls.pop();
                        if tmp.is_none() {
                            let wait_time = rand::thread_rng().gen_range(1..5);
                            tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                            pending_urls = client
                                .get_pending_urls(
                                    num_of_threads - worker_rx.len(),
                                    comic_url.clone(),
                                )
                                .await;
                            continue;
                            // break;
                        }
                        tmp.unwrap()
                    };
                    fetching_url.lock().unwrap().insert(pending_url.to_string());
                    worker_tx
                        .send(types::thread_message::ThreadMessage::Start(
                            pending_url.to_string(),
                            0,
                        ))
                        .await
                        .unwrap();
                }
            }
            _ => {}
        }
    }
    log::info!("finished");
    while worker_rx.len() > 0 {
        let _ = dbg!(worker_rx.recv().await);
    }
    for w in &workers {
        let _ = worker_tx
            .send(types::thread_message::ThreadMessage::Exited(w.id))
            .await;
    }
    // for w in workers {
    //     w.join_handle.await.unwrap();
    //     log::info!("worker joined");
    // }
    let _ = async {
        let client = db::DbUtils::new().await.unwrap();
        // update fetched = false for fetching url
        let mut update_data = vec![];
        for url in &fetching_url.clone().lock().unwrap().clone() {
            // update_data.push(client.urls().update(
            //     prisma::urls::UniqueWhereParam::UrlEquals(url.clone()),
            //     vec![prisma::urls::fetched::set(false)],
            // ));
            update_data.push(client.update_url_doc_daft(
                url.to_string(),
                vec![db::UpdateUrlDocFields::Fetched(false)],
            ));
        }
        let _ = client._batch(update_data).await;
    };
    process::exit(0);
    // Ok(())
}

mod tests {

    #[tokio::test]
    async fn test_request_cf_reqwest() {
        let client = reqwest::Client::new();
        let resp = client.get("https://nowsecure.nl").send().await;
        assert!(resp.is_ok());
    }
    #[tokio::test]
    async fn test_request_cf_headless_chrome(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use undetected_chromedriver::chrome;
        let driver = chrome().await?;
        driver.goto("https://nowsecure.nl").await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let png = driver.screenshot_as_png().await?;
        let image = image::load_from_memory(&png)?;
        image.save("image.png")?;
        Ok(())
    }
}
