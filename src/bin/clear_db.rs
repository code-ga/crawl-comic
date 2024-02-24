mod prisma_client;

#[tokio::main]
async fn main() {
    let client: prisma_client::PrismaClient = prisma_client::PrismaClient::_builder()
        .build()
        .await
        .unwrap();
    client.urls().delete_many(vec![]).exec().await.unwrap();
    client.chapter().delete_many(vec![]).exec().await.unwrap();
    client.comic().delete_many(vec![]).exec().await.unwrap();
}
