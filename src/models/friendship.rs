pub enum FriendshipStatus {
    Pending,
    Accepted,
    Declined,
    Blocked,
}

pub struct Friendship {
    friendship_id: String,
    initiator_id: String,
    recipient_id: String,
    status: String,
}