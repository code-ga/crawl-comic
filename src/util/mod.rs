use crate::prisma::{self, PrismaClient};
use rand::seq::SliceRandom;

pub async fn get_pending_urls(
    client: &PrismaClient,
    num_of_url: usize,
    now_fetched_url: String,
) -> Vec<String> {
    let (_, urls) = client
        ._batch((
            client.urls().update_many(
                vec![prisma::urls::url::equals(now_fetched_url.clone())],
                vec![
                    prisma::urls::fetched::set(true),
                    prisma::urls::fetching::set(false),
                ],
            ),
            client
                .urls()
                .find_many(vec![
                    prisma::urls::fetching::equals(false),
                    prisma::urls::fetched::equals(false),
                ])
                .take(num_of_url.try_into().unwrap()),
        ))
        .await
        .unwrap();
    let mut result = std::collections::HashSet::new();
    for u in urls {
        client
            .urls()
            .update_many(
                vec![prisma::urls::url::equals(u.url.clone())],
                vec![
                    prisma::urls::fetching::set(true),
                    prisma::urls::fetched::set(false),
                ],
            )
            .exec()
            .await
            .unwrap();
        result.insert(u.url.to_string());
    }
    return result.into_iter().collect::<Vec<_>>();
}

pub async fn filters_urls(url: String, client: &PrismaClient) -> Option<String> {
    // let client = client.lock().await;
    // let url = process_url(&urls);
    // if url.is_none() {
    //     return None;
    // }
    let url_in_db = {
        let tmp = client
            .urls()
            .find_first(vec![prisma::urls::url::equals(url.clone())])
            .exec()
            .await;
        if tmp.is_err() {
            return None;
        }
        tmp.unwrap()
    };
    if url_in_db.is_some() {
        return None;
    } else {
        return Some(url.clone());
    }
}

pub async fn get_proxy(client: &PrismaClient) -> Option<prisma::proxy::Data> {
    let proxies = client.proxy().find_many(vec![]).exec().await.unwrap();
    if proxies.is_empty() {
        return None;
    }
    let proxy = proxies.choose(&mut rand::thread_rng()).unwrap();
    return Some(proxy.clone());
}
