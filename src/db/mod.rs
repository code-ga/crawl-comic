use crate::prisma;

use prisma_client_rust::raw;
use rand::Rng;

pub struct DbUtils {
    pub prisma: crate::prisma::PrismaClient,
}

impl DbUtils {
    pub async fn new() -> Result<DbUtils, Box<dyn std::error::Error + Send + Sync>> {
        Ok(DbUtils {
            prisma: match prisma::PrismaClient::_builder().build().await {
                Ok(prisma) => prisma,
                Err(e) => {
                    return Err(Box::new(e));
                }
            },
        })
    }
}

pub enum UpdateUrlDocFields {
    Fetched(bool),
    Fetching(bool),
    IsError(bool),
}

impl UpdateUrlDocFields {
    pub fn to_prisma_set(&self) -> prisma::urls::SetParam {
        match self {
            UpdateUrlDocFields::Fetched(value) => prisma::urls::SetParam::SetFetched(*value),
            UpdateUrlDocFields::Fetching(value) => prisma::urls::SetParam::SetFetching(*value),
            UpdateUrlDocFields::IsError(value) => prisma::urls::SetParam::SetIsError(*value),
        }
    }
}

// urls services
impl DbUtils {
    pub async fn find_url_doc_by_url(
        &self,
        url: &str,
    ) -> Result<Option<prisma::urls::Data>, prisma_client_rust::queries::QueryError> {
        self.prisma
            .urls()
            .find_unique(prisma::urls::UniqueWhereParam::UrlEquals(url.to_string()))
            .exec()
            .await
    }
    pub async fn create_url_doc(
        &self,
        url: &str,
    ) -> Result<prisma::urls::Data, prisma_client_rust::queries::QueryError> {
        log::debug!("create url {}", url);
        self.prisma
            .urls()
            .create(url.to_string(), vec![])
            .exec()
            .await
    }

    pub async fn update_url_doc(
        &self,
        url: String,
        update_field: Vec<UpdateUrlDocFields>,
    ) -> Result<prisma::urls::Data, prisma_client_rust::queries::QueryError> {
        self.prisma
            .urls()
            .update(
                prisma::urls::UniqueWhereParam::UrlEquals(url.to_string()),
                update_field.iter().map(|x| x.to_prisma_set()).collect(),
            )
            .exec()
            .await
    }
    pub fn update_url_doc_daft(
        &self,
        url: String,
        update_field: Vec<UpdateUrlDocFields>,
    ) -> prisma::urls::Update {
        self.prisma.urls().update(
            prisma::urls::UniqueWhereParam::UrlEquals(url.to_string()),
            update_field.iter().map(|x| x.to_prisma_set()).collect(),
        )
    }
    pub async fn filters_urls(&self, url: String) -> Option<String> {
        // let url = process_url(&urls);
        // if url.is_none() {
        //     return None;
        // }
        let url_in_db = {
            let tmp = self
                .prisma
                .urls()
                .find_unique(prisma::urls::UniqueWhereParam::UrlEquals(url.clone()))
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
}

pub type DbProxyData = prisma::proxy::Data;
// proxy service
impl DbUtils {
    pub async fn get_proxy(&self) -> Option<DbProxyData> {
        let proxies_count = self.prisma.proxy().count(vec![]).exec().await.unwrap();
        let mut skip_proxies = if proxies_count == 0 {
            return None;
        } else {
            rand::thread_rng().gen_range(0..(proxies_count))
        };
        if skip_proxies == proxies_count && skip_proxies != 0 {
            skip_proxies -= 1;
        }
        let proxies = self
            .prisma
            .proxy()
            .find_first(vec![])
            .skip(skip_proxies)
            .exec()
            .await
            .unwrap();
        // dbg!(&proxies);
        return proxies;
    }
}

// util
impl DbUtils {
    pub async fn _batch<'batch, T: ::prisma_client_rust::BatchContainer<'batch, Marker>, Marker>(
        &self,
        queries: T,
    ) -> ::prisma_client_rust::Result<
        <T as ::prisma_client_rust::BatchContainer<'batch, Marker>>::ReturnType,
    > {
        self.prisma._batch(queries).await
    }
    pub async fn get_pending_urls(
        &self,
        num_of_url: usize,
        now_fetched_url: String,
    ) -> Vec<String> {
        let (_, urls) = self
            .prisma
            ._batch((
                self.prisma.urls().update(
                    prisma::urls::UniqueWhereParam::UrlEquals(now_fetched_url.clone()),
                    vec![
                        prisma::urls::fetched::set(true),
                        prisma::urls::fetching::set(false),
                    ],
                ),
                self.prisma._query_raw::<prisma::urls::Data>(raw!(
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
            self.prisma
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
}

// comic type service
#[derive(Debug, Clone)]
pub enum UpdateComicDocField {
    Name(String),
    PythonFetchInfo(bool),
    Content(Option<String>),
    ThumbnailUrl(Option<String>),
    AnotherName(Vec<String>),
    Author(serde_json::Value),
    Source(serde_json::Value),
    TranslatorTeam(serde_json::Value),
    PostedBy(serde_json::Value),
    Genre(serde_json::Value),
    Status(String),
}

impl UpdateComicDocField {
    pub fn to_prisma_set(&self) -> prisma::comic::SetParam {
        match &self {
            &UpdateComicDocField::Name(value) => {
                prisma::comic::SetParam::SetName(value.to_string())
            }
            &UpdateComicDocField::PythonFetchInfo(value) => {
                prisma::comic::SetParam::SetPythonFetchInfo(*value)
            }
            &UpdateComicDocField::Content(value) => {
                prisma::comic::SetParam::SetContent(value.clone())
            }
            &UpdateComicDocField::ThumbnailUrl(value) => {
                prisma::comic::SetParam::SetThumbnail(value.clone())
            }
            &UpdateComicDocField::AnotherName(value) => {
                prisma::comic::SetParam::SetAnotherName(value.to_vec())
            }
            &UpdateComicDocField::Author(value) => {
                prisma::comic::SetParam::SetAuthor(value.clone())
            }
            &UpdateComicDocField::Source(value) => {
                prisma::comic::SetParam::SetSource(value.clone())
            }
            &UpdateComicDocField::TranslatorTeam(value) => {
                prisma::comic::SetParam::SetTranslatorTeam(value.clone())
            }
            &UpdateComicDocField::PostedBy(value) => {
                prisma::comic::SetParam::SetPostedBy(value.clone())
            }
            &UpdateComicDocField::Genre(value) => prisma::comic::SetParam::SetGenre(value.clone()),
            &UpdateComicDocField::Status(value) => {
                prisma::comic::SetParam::SetStatus(value.to_string())
            }
        }
    }
}

// comic service
impl DbUtils {
    pub async fn comic_by_url(
        &self,
        url: String,
    ) -> Result<Option<prisma::comic::Data>, prisma_client_rust::queries::QueryError> {
        self.prisma
            .comic()
            .find_unique(prisma::comic::UniqueWhereParam::UrlEquals(url.to_string()))
            .exec()
            .await
    }
    pub async fn create_empty_comic(
        &self,
        url: String,
    ) -> Result<prisma::comic::Data, prisma_client_rust::queries::QueryError> {
        self.prisma.comic().create(url, vec![]).exec().await
    }
    pub async fn update_comic_by_url(
        &self,
        url: String,
        update_field: Vec<UpdateComicDocField>,
    ) -> Result<prisma::comic::Data, prisma_client_rust::queries::QueryError> {
        self.prisma
            .comic()
            .update(
                prisma::comic::UniqueWhereParam::UrlEquals(url.to_string()),
                update_field.iter().map(|x| x.to_prisma_set()).collect(),
            )
            .exec()
            .await
    }
}

// chapter type service
#[derive(Debug, Clone)]
pub enum ChapterUpdateField {
    Url(String),
    Images(Vec<String>),
    ServerImage(Vec<serde_json::Value>),
    CreatedDate(String),
    Index(i32),
}

impl ChapterUpdateField {
    pub fn to_prisma_set(&self) -> prisma::chapter::SetParam {
        match &self {
            &ChapterUpdateField::Url(value) => prisma::chapter::SetParam::SetUrl(value.to_string()),
            &ChapterUpdateField::Images(value) => {
                prisma::chapter::SetParam::SetImages(value.to_vec())
            }
            &ChapterUpdateField::ServerImage(value) => {
                prisma::chapter::SetParam::SetServerImage(value.to_vec())
            }
            &ChapterUpdateField::CreatedDate(value) => {
                prisma::chapter::SetParam::SetCreatedDate(value.to_string())
            }
            &ChapterUpdateField::Index(value) => prisma::chapter::SetParam::SetIndex(*value),
        }
    }
}

pub type DaftChapter = (
    String,
    String,
    String,
    String,
    Vec<prisma::chapter::SetParam>,
);

// chapter service
impl DbUtils {
    pub async fn chapter_by_url(
        &self,
        url: String,
    ) -> Result<Option<prisma::chapter::Data>, prisma_client_rust::queries::QueryError> {
        self.prisma
            .chapter()
            .find_unique(prisma::chapter::UniqueWhereParam::UrlEquals(
                url.to_string(),
            ))
            .exec()
            .await
    }
    pub async fn update_chapter_by_url(
        &self,
        url: String,
        update_field: Vec<ChapterUpdateField>,
    ) -> Result<i64, prisma_client_rust::queries::QueryError> {
        self.prisma
            .chapter()
            .update_many(
                vec![crate::prisma::chapter::url::equals(url.to_string())],
                update_field.iter().map(|x| x.to_prisma_set()).collect(),
            )
            .exec()
            .await
    }
    pub fn daft_update_chapter_by_url(
        &self,
        url: String,
        update_field: Vec<ChapterUpdateField>,
    ) -> prisma::chapter::Update {
        self.prisma.chapter().update(
            prisma::chapter::UniqueWhereParam::UrlEquals(url.to_string()),
            update_field.iter().map(|x| x.to_prisma_set()).collect(),
        )
    }
    pub async fn create_empty_chapter(
        &self,
        name: String,
        url: String,
        created_date: String,
        comic_id: String,
        index: Option<i32>,
    ) -> Result<prisma::chapter::Data, prisma_client_rust::queries::QueryError> {
        self.prisma
            .chapter()
            .create(
                name,
                url,
                created_date,
                prisma::comic::UniqueWhereParam::IdEquals(comic_id.to_string()),
                {
                    if index.is_none() {
                        vec![]
                    } else {
                        vec![prisma::chapter::index::set(index.unwrap())]
                    }
                },
            )
            .exec()
            .await
    }
    pub fn create_empty_chapter_dafter(
        &self,
        name: String,
        url: String,
        created_date: String,
        comic_id: String,
        index: Option<i32>,
    ) -> DaftChapter {
        prisma::chapter::create_unchecked(name, url, comic_id.to_string(), created_date, {
            if index.is_none() {
                vec![]
            } else {
                vec![prisma::chapter::index::set(index.unwrap())]
            }
        })
    }

    pub async fn create_many_chapters(
        &self,
        chapters: Vec<DaftChapter>,
    ) -> Result<i64, prisma_client_rust::queries::QueryError> {
        self.prisma.chapter().create_many(chapters).exec().await
    }
}
