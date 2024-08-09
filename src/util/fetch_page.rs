use crate::db::{DbUtils, UpdateUrlDocFields};

pub async fn get_http_client(
    client: &DbUtils,
) -> Result<
    (reqwest::Client, Option<crate::db::DbProxyData>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let proxy = client.get_proxy().await;
    let mut http_client = reqwest::ClientBuilder::new();
    if proxy.is_some() {
        let proxy = proxy.clone().unwrap();
        log::info!("using proxy {} - {}", proxy.id, proxy.url);
        http_client = http_client.proxy({
            let mut client_proxy = reqwest::Proxy::all(proxy.url.to_string().trim().to_string())?;
            if proxy.username.is_some() && proxy.password.is_some() {
                client_proxy =
                    client_proxy.basic_auth(&proxy.username.unwrap(), &proxy.password.unwrap())
            }
            client_proxy
        });
    }
    let nettruyen_dns = vec![
        std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(172, 67, 136, 13)),
            443,
        ),
        std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(104, 21, 46, 67)),
            443,
        ),
    ];
    Ok((
        http_client
            .resolve_to_addrs("nettruyenbb.com", nettruyen_dns.clone().as_slice())
            .resolve_to_addrs("www.nettruyenbb.com", nettruyen_dns.clone().as_slice())
            .build()?,
        proxy,
    ))
}

async fn fetch_page_with_reqwest(
    client: &DbUtils,
    hostname: &str,
    worker_id: usize,
    url: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let (http_client, _) = get_http_client(client).await?;

    let mut rep = http_client
        .get(url.clone())
        .header("User-Agent", "Mozilla/5.0");
    if hostname.contains("blogtruyenmoi.com") {
        rep = rep.header("Referrer", format!("https://{}/", hostname.to_string()));
    }
    let resp = rep.send().await;
    if let Err(e) = resp {
        log::info!(
            "worker {} failed {} fetching error {:#?}",
            worker_id,
            url.to_string(),
            e.to_string()
        );
        return Err(Box::new(e));
    }
    let resp_unwrap = resp.unwrap();

    if resp_unwrap.status().is_success() == false && resp_unwrap.status().as_u16().eq(&404) == false
    {
        if resp_unwrap.status().as_u16().eq(&429) {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        log::error!(
            "worker {} failed {} status : {}",
            worker_id,
            url.to_string(),
            resp_unwrap.status()
        );
        // dbg!(&resp_unwrap.text().await);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("worker {} failed {}", worker_id, url,),
        )));
    } else if resp_unwrap.status().as_u16().eq(&404) {
        {
            // let tmp = client
            //     .urls()
            //     .update(
            //         prisma::urls::UniqueWhereParam::UrlEquals(url.clone()),
            //         vec![
            //             prisma::urls::fetched::set(true),
            //             prisma::urls::fetching::set(false),
            //         ],
            //     )
            //     .exec()
            //     .await;
            let tmp = client
                .update_url_doc(
                    url.to_string(),
                    vec![
                        UpdateUrlDocFields::Fetched(true),
                        UpdateUrlDocFields::Fetching(false),
                    ],
                )
                .await;
            if let Err(e) = tmp {
                return Err(Box::new(e));
            }
        };
        log::info!(
            "worker {} done {} status : {}",
            worker_id,
            url.to_string(),
            resp_unwrap.status()
        );
        return Ok("".to_string());
    }
    let html = {
        let tmp = resp_unwrap.text().await;
        if let Err(e) = tmp {
            return Err(Box::new(e));
        }
        tmp.unwrap()
    };
    Ok(html)
}

// async fn fetch_page_with_headless_browser(
//     _client: &DbUtils,
//     hostname: &str,
//     _worker_id: usize,
//     url: String,
// ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
//     log::info!("fetching with headless {}", url);
//     let driver = undetected_chromedriver::chrome().await.unwrap();
//     driver.goto(&url).await?;
//     let mut num_of_tries = 0;
//     if NETTRUYEN_HOSTS.contains(&hostname) {
//         loop {
//             if num_of_tries > 10 {
//                 return Err(Box::new(std::io::Error::new(
//                     std::io::ErrorKind::Other,
//                     format!("can't try more than 10 times check page {}", url),
//                 )));
//             }
//             match driver
//                 .find(undetected_chromedriver::By::XPath(
//                     "/html/body/form/main/div[2]",
//                 ))
//                 .await
//             {
//                 Err(e) => match e {
//                     undetected_chromedriver::WebDriverError::NoSuchElement(_) => {
//                         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//                         num_of_tries += 1;
//                         continue;
//                     }
//                     _ => {
//                         return Err(Box::new(e));
//                     }
//                 },
//                 Ok(_) => break,
//             }
//         }
//     } else if hostname.contains("blogtruyenmoi.com") {
//         loop {
//             if num_of_tries > 10 {
//                 return Err(Box::new(std::io::Error::new(
//                     std::io::ErrorKind::Other,
//                     format!("can't try more than 10 times check page {}", url),
//                 )));
//             }
//             match driver
//                 .find(undetected_chromedriver::By::XPath(
//                     if url.starts_with("https://blogtruyenmoi.com/c") {
//                         r#"//*[@id="readonline"]/section[1]"#
//                     } else if url.starts_with("https://blogtruyenmoi.com/ajax/Search/") {
//                         "/html/body/div[1]"
//                     } else {
//                         r#"//*[@id="banner"]"#
//                     },
//                 ))
//                 .await
//             {
//                 Err(e) => match e {
//                     undetected_chromedriver::WebDriverError::NoSuchElement(_) => {
//                         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//                         num_of_tries += 1;
//                         continue;
//                     }
//                     _ => {
//                         return Err(Box::new(e));
//                     }
//                 },
//                 Ok(_) => break,
//             }
//         }
//     } else {
//         unimplemented!("not yet implemented");
//     }
//     let html = driver.source().await?;
//     driver.quit().await?;
//     Ok(html)
// }

pub async fn fetch_page(
    client: &DbUtils,
    hostname: &str,
    worker_id: usize,
    url: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // if let Ok(html) = fetch_page_with_reqwest(client, hostname, worker_id, url.clone()).await {
    //     return Ok(html);
    // } else {
    //     return fetch_page_with_headless_browser(client, hostname, worker_id, url.clone()).await;
    // }
    return fetch_page_with_reqwest(client, hostname, worker_id, url).await
}