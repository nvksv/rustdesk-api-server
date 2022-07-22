use utils::{UserId};

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: UserId,
    pub active: bool,
    pub admin: bool,
    pub username: String,
    pub password: String,
    pub address_book: String,
}

