use rocket::{
    serde::{Serialize, Deserialize},
};

use crate::{
    tokens::Token,
};

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub id: String,
    pub uuid: String,
}

#[derive(Serialize, Debug)]
pub struct UserInfo {
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct LoginReply {
    pub user: UserInfo,
    pub access_token: Token,
}

#[derive(Serialize, Debug)]
pub struct LogoutReply {
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct CurrentUserRequest {
    pub id: String,
    pub uuid: String,
}

#[derive(Serialize, Debug)]
pub struct CurrentUserResponse {
    pub error: bool,
    #[serde(flatten)]
    pub data: UserInfo,
}

#[derive(Deserialize, Debug)]
pub struct AbRequest {
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct AuditRequest {
    #[serde(default)]
    #[serde(rename = "Id")]
    pub id_: usize,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub uuid: String,
}

// {
//    peers: [{id: "abcd", username: "", hostname: "", platform: "", alias: "", tags: ["", "", ...]}, ...],
//    tags: [],
// }

#[derive(Serialize, Debug)]
pub struct AbPeer {
    pub id: String,
}

#[derive(Serialize, Debug)]
pub struct Ab {
    pub tags: Vec<String>,
    pub peers: Vec<AbPeer>,
}


#[derive(Serialize, Debug)]
pub struct AbGetResponse {
    pub error: bool,
    pub updated_at: String,
    pub data: String,
}