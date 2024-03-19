use crate::prisma::{self, PrismaClient};
use prisma_client_rust::raw;
use rand::Rng;
use regex::Regex;

pub async fn get_pending_urls(
    client: &PrismaClient,
    num_of_url: usize,
    now_fetched_url: String,
) -> Vec<String> {
    let (_, urls) = client
        ._batch((
            client.urls().update(
                prisma::urls::UniqueWhereParam::UrlEquals(now_fetched_url.clone()),
                vec![
                    prisma::urls::fetched::set(true),
                    prisma::urls::fetching::set(false),
                ],
            ),
            // client
            //     .urls()
            //     .find_many(vec![
            //         prisma::urls::fetching::equals(false),
            //         prisma::urls::fetched::equals(false),
            //     ])
            //     .take(num_of_url.try_into().unwrap()),
            client._query_raw::<prisma::urls::Data>(raw!(
                "SELECT * FROM \"public\".\"Urls\" WHERE (fetched={} AND fetching={}) LIMIT {}",
                prisma_client_rust::PrismaValue::Boolean(false),
                prisma_client_rust::PrismaValue::Boolean(false),
                num_of_url.try_into().unwrap()
            )),
        ))
        .await
        .unwrap();
    let mut result = std::collections::HashSet::new();
    for u in urls {
        client
            .urls()
            .update(
                prisma::urls::UniqueWhereParam::UrlEquals(u.url.clone()),
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
    let proxies_count = client.proxy().count(vec![]).exec().await.unwrap();
    let mut skip_proxies = if proxies_count == 0 {
        return None;
    } else {
        rand::thread_rng().gen_range(0..(proxies_count))
    };
    if skip_proxies == proxies_count && skip_proxies != 0 {
        skip_proxies -= 1;
    }
    let proxies = client
        .proxy()
        .find_first(vec![])
        .skip(skip_proxies)
        .exec()
        .await
        .unwrap();
    // dbg!(&proxies);
    return proxies;
}

pub fn get_host(url: &str) -> Option<String> {
    let re = Regex::new(r#"https?://([^/]+)"#).unwrap();
    let cap = re.captures(url);
    if cap.is_none() {
        return None;
    }
    return Some(cap.unwrap()[1].to_string());
}
