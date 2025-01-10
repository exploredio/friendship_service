use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use crate::neo4j::{create_connection, get_nodes, respond_to_friend_request, send_friend_request};
use dotenv::dotenv;

mod neo4j;
pub mod models{
    pub mod friendship;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let graph = Arc::new(create_connection().await.expect("Failed to connect to Neo4j"));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(graph.clone()))
            .service(get_nodes)
            .service(send_friend_request)
            .service(respond_to_friend_request)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}