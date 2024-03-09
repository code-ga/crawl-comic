use prisma::PrismaClient;
use rand::Rng;

use types::thread_message::ThreadMessage;

mod modules;
pub mod prisma;
mod types;
mod util;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client: PrismaClient = PrismaClient::_builder().build().await.unwrap();
    let num_of_threads = 10;
    let init_url =
        "https://blogtruyenmoi.com/ajax/Search/AjaxLoadListManga?key=tatca&orderBy=3&p=1"
            .to_string();
    // worker to main channel
    let (main_tx, main_rx) = async_channel::bounded::<ThreadMessage>(num_of_threads + 5);
    // main to worker channel
    let (worker_tx, worker_rx) = async_channel::bounded::<ThreadMessage>(num_of_threads + 5);
    // let rx = Arc::new(Mutex::new(worker_rx));
    let mut workers = Vec::new();
    for i in 0..num_of_threads {
        println!("spawn {}", i);
        let tx = main_tx.clone();
        let rx = worker_rx.clone();
        let worker = tokio::spawn(async move {
            modules::blogtruyenmoi::thread_worker(tx, rx, i).await;
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

    loop {
        let job = main_rx.recv().await.unwrap();
        if worker_rx.is_empty() {
            let wait_time = rand::thread_rng().gen_range(1..5);
            tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
            let mut pending_url = util::get_pending_urls(
                &client,
                num_of_threads + 5 - worker_rx.len(),
                "".to_string(),
            )
            .await;
            while !worker_rx.is_full() {
                let pending_url = {
                    let tmp = pending_url.pop();
                    if tmp.is_none() {
                        let wait_time = rand::thread_rng().gen_range(1..5);
                        tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                        pending_url = util::get_pending_urls(
                            &client,
                            num_of_threads + 5 - worker_rx.len(),
                            "".to_string(),
                        )
                        .await;
                        continue;
                    }
                    tmp.unwrap()
                };
                worker_tx
                    .send(types::thread_message::ThreadMessage::Start(
                        pending_url.to_string(),
                        0,
                    ))
                    .await
                    .unwrap();
            }
        }
        match job {
            ThreadMessage::Stop(id) => {
                // spawn new worker and replace old
                let tx = main_tx.clone();
                let rx = worker_rx.clone();
                let worker = tokio::spawn(async move {
                    modules::blogtruyenmoi::thread_worker(tx, rx, id).await;
                });
                workers[id].join_handle = worker;
            }
            ThreadMessage::Retry(url, i) => {
                let wait_time = rand::thread_rng().gen_range(1..5);
                tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                if i >= 10 {
                    client
                        .urls()
                        .update_many(
                            vec![prisma::urls::url::equals(url.to_string())],
                            vec![
                                prisma::urls::is_error::set(true),
                                prisma::urls::fetched::set(true),
                            ],
                        )
                        .exec()
                        .await
                        .unwrap();
                    let mut pending_urls = util::get_pending_urls(
                        &client,
                        num_of_threads + 5 - worker_rx.len(),
                        url.clone(),
                    )
                    .await;
                    // dbg!(&pending_urls);
                    while !worker_rx.is_full() {
                        let pending_url = {
                            let tmp = pending_urls.pop();
                            if tmp.is_none() {
                                // let wait_time = rand::thread_rng().gen_range(1..5);
                                // tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                                // pending_urls = util::get_pending_urls(
                                //     &client,
                                //     num_of_threads + 5 - worker_rx.len(),
                                //     url.clone(),
                                // )
                                // .await;
                                // continue;
                                break;
                            }
                            tmp.unwrap()
                        };
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
                println!("retry {}", url);
                worker_tx
                    .send(types::thread_message::ThreadMessage::Start(url, i + 1))
                    .await
                    .unwrap();
            }
            ThreadMessage::Done(worker_pending_url, comic_url, _) => {
                for u in worker_pending_url {
                    let url = util::filters_urls(u, &client).await;
                    if url.is_none() {
                        continue;
                    };
                    client
                        .urls()
                        .create(url.unwrap().to_string(), vec![])
                        .exec()
                        .await
                        .unwrap();
                }

                let wait_time = rand::thread_rng().gen_range(1..5);
                tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                // sleep 1s
                let mut pending_urls = util::get_pending_urls(
                    &client,
                    num_of_threads + 5 - worker_rx.len(),
                    comic_url.clone(),
                )
                .await;
                // dbg!(&pending_urls);
                while !worker_rx.is_full() {
                    let pending_url = {
                        let tmp = pending_urls.pop();
                        if tmp.is_none() {
                            // let wait_time = rand::thread_rng().gen_range(1..5);
                            // tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
                            // pending_urls = util::get_pending_urls(
                            //     &client,
                            //     num_of_threads + 5 - worker_rx.len(),
                            //     comic_url.clone(),
                            // )
                            // .await;
                            // continue;
                            break;
                        }
                        tmp.unwrap()
                    };
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

    // Ok(())
}
