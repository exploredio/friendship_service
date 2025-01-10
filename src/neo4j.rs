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

    WITH u1, u2,
         CASE
             WHEN u1 = u2 THEN "You cannot send a follow request to yourself"
             WHEN EXISTS((u1)-[:BLOCKED]->(u2)) THEN "You have blocked this user"
             WHEN EXISTS((u2)-[:BLOCKED]->(u1)) THEN "This user has blocked you"
             WHEN EXISTS((u1)-[:PENDING]->(u2)) THEN "You have already sent a follow request to this user"
             WHEN EXISTS((u2)-[:PENDING]->(u1)) THEN "This user has already sent you a follow request"
             WHEN EXISTS((u1)-[:ACCEPTED]->(u2)) THEN "You are already friends with this user"
             WHEN EXISTS((u2)-[:ACCEPTED]->(u1)) THEN "You are already friends with this user"
             ELSE "OK"
         END AS message

    WITH u1, u2, message
    FOREACH (_ IN CASE WHEN message = "OK" THEN [1] ELSE [] END |
        MERGE (u1)-[:PENDING {datetime: datetime()}]->(u2)
    )

    RETURN message
    "#;

    let mut result = graph
        .execute(query(cypher)
                     .param("initiator_id", initiator_id)
                     .param("recipient_id", recipient_id),
        )
        .await
        .unwrap();
    match result.next().await {
        Ok(Some(row)) => {
            let message: String = row.get("message").unwrap();

            if message == "OK" {
                HttpResponse::Ok().body("Follow request sent")
            } else {
                HttpResponse::BadRequest().body(message)
            }
        }
        Ok(None) => HttpResponse::InternalServerError().body("No rows returned"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
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

    let cypher = if status == "BLOCKED" {
        r#"MATCH (u1:User {id: $initiator_id}), (u2:User {id: $recipient_id})
        OPTIONAL MATCH (u1)-[r]->(u2)
        DELETE r
        WITH u1, u2
        CALL apoc.create.relationship(u1, $status, {}, u2) YIELD rel as new_r
        RETURN new_r
        "#
    } else {
        r#"MATCH (u1:User {id: $initiator_id})-[r:PENDING]->(u2:User {id: $recipient_id})
        CALL apoc.create.relationship(u1, $status, {}, u2) YIELD rel as new_r
        DELETE r
        RETURN new_r
        "#
    };


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