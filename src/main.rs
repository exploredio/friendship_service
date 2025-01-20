use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use crate::neo4j::{create_connection, get_friendships_by_user_id, respond_to_friend_request, send_friend_request};
use dotenv::dotenv;

mod neo4j;
pub mod models{
    pub mod friendship;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenv().ok();

    let graph = Arc::new(create_connection().await.expect("Failed to connect to Neo4j"));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(graph.clone()))
            .service(send_friend_request)
            .service(respond_to_friend_request)
            .service(get_friendships_by_user_id)
    })
    .bind("0.0.0.0:8086")?
    .run()
    .await
}