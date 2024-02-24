use prisma::PrismaClient;

use types::thread_message::ThreadMessage;

mod modules;
mod prisma;
mod types;
mod util;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client: PrismaClient = PrismaClient::_builder().build().await.unwrap();
    let num_of_threads = 10;
    let init_url = "https://blogtruyenmoi.com".to_string();
    // worker to main channel
    let (main_tx, main_rx) = async_channel::bounded::<ThreadMessage>(num_of_threads);
    // main to worker channel
    let (worker_tx, worker_rx) = async_channel::bounded::<ThreadMessage>(num_of_threads);
    // let rx = Arc::new(Mutex::new(worker_rx));
    let mut workers = Vec::new();
    for i in 0..num_of_threads {
        println!("spawn {}", i);
        let tx = main_tx.clone();
        let rx = worker_rx.clone();
        let proxy = util::get_proxy(&client).await;

        let worker = tokio::spawn(async move {
            modules::blogtruyenmoi::thread_worker(tx, rx, i, proxy.clone()).await;
        });

        workers.push(types::Worker {
            id: i,
            join_handle: worker,
        });
    }

    worker_tx
        .send(types::thread_message::ThreadMessage::Start(init_url))
        .await
        .unwrap();

    loop {
        let job = main_rx.recv().await.unwrap();
        match job {
            ThreadMessage::Stop(id) => {
                // spawn new worker and replace old
                let tx = main_tx.clone();
                let rx = worker_rx.clone();
                let proxy = util::get_proxy(&client).await;
                let worker = tokio::spawn(async move {
                    modules::blogtruyenmoi::thread_worker(tx, rx, id, proxy.clone()).await;
                });
                workers[id].join_handle = worker;
            }
            ThreadMessage::Retry(url) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                worker_tx
                    .send(types::thread_message::ThreadMessage::Start(url))
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

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                // sleep 1s
                let mut pending_urls = util::get_pending_urls(
                    &client,
                    num_of_threads - worker_rx.len(),
                    comic_url.clone(),
                )
                .await;
                // dbg!(&pending_urls);
                while !worker_rx.is_full() {
                    let pending_url = {
                        let tmp = pending_urls.pop();
                        if tmp.is_none() {
                            // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            pending_urls = util::get_pending_urls(
                                &client,
                                num_of_threads - worker_rx.len(),
                                comic_url.clone(),
                            )
                            .await;
                            continue;
                        }
                        tmp.unwrap()
                    };
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    worker_tx
                        .send(types::thread_message::ThreadMessage::Start(
                            pending_url.to_string(),
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
