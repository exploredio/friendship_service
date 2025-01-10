use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Friendship {
    pub initiator_id: String,
    pub recipient_id: String,
    pub status: String,
}