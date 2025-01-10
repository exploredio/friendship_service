use neo4rs::{query, Error, Graph};
use std::env;
use std::sync::Arc;
use actix_web::{post, get, put, web, Responder, HttpResponse};
use crate::models::friendship::Friendship;

pub async fn create_connection() -> Result<Graph, Error> {
    let uri = env::var("NEO4J_URI").expect("NEO4J_URI must be set");
    let username = env::var("NEO4J_USERNAME").expect("NEO4J_USERNAME must be set");
    let password = env::var("NEO4J_PASSWORD").expect("NEO4J_PASSWORD must be set");

    Graph::new(&uri, &username, &password).await
}

#[get("/nodes")]
async fn get_nodes(graph: web::Data<Arc<Graph>>) -> impl Responder {
    let mut result = graph.execute(query("MATCH (n) RETURN n")).await.unwrap();
    let mut nodes = vec![];

    while let Ok(Some(row)) = result.next().await {
        let node: neo4rs::Node = row.get("n").unwrap();
        nodes.push(node.id());
    }

    format!("Node IDs: {:?}", nodes)
}

#[post("/friendships/initiate")]
async fn send_friend_request(
    friendship: web::Json<Friendship>,
    graph: web::Data<Arc<Graph>>,
) -> impl Responder {
    let Friendship {
        initiator_id,
        recipient_id,
        ..
    } = friendship.into_inner();

    let cypher = r#"
        MERGE (u1:User {id: $initiator_id})
        MERGE (u2:User {id: $recipient_id})
        CREATE (u1)-[:PENDING {datetime: datetime()}]->(u2)
    "#;

    let result = graph
        .run(query(cypher)
                 .param("initiator_id", initiator_id)
                 .param("recipient_id", recipient_id),
        )
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Follow request sent successfully"),
        Err(err) => {
            eprintln!("Error sending follow request: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to send follow request")
        }
    }
}

#[put("/friendships/respond")]
async fn respond_to_friend_request(
    friendship: web::Json<Friendship>,
    graph: web::Data<Arc<Graph>>,
) -> impl Responder {
    let Friendship {
        initiator_id,
        recipient_id,
        status,
    } = friendship.into_inner();

    let status: &str = match status.to_lowercase().as_str() {
        "accepted" => "ACCEPTED",
        "declined" => "DECLINED",
        "blocked" => "BLOCKED",
        _ => return HttpResponse::BadRequest().body("Invalid friendship status"),
    };

    let cypher = r#"
        MATCH (u1:User {id: $initiator_id})-[r:PENDING]->(u2:User {id: $recipient_id})
        SET r = $status
        RETURN r
    "#;

    let result = graph
        .execute(query(cypher)
            .param("initiator_id", initiator_id)
            .param("recipient_id", recipient_id)
            .param("status", status)
        )
        .await;
    match result {
        Ok(mut rows) => {
            if rows.next().await.unwrap().is_none() {
                HttpResponse::NotFound().body("Friendship request not found")
            } else {
                HttpResponse::Ok().body(format!("Friendship {}", status.to_lowercase()))
            }
        }
        Err(err) => {
            eprintln!("Error executing query: {}", err);
            HttpResponse::InternalServerError().body("Failed to update friendship status")
        }
    }
}