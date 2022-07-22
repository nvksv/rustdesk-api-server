use rocket::{
    request::{Request, FromRequest, Outcome},
    State, 
    outcome::try_outcome,
};
use utils::{unwrap_or_return, Token, BearerToken};
use crate::{
    SessionId, UserId, 
    state::ApiState,
};


#[derive(Debug)]
pub struct AuthenticatedUser {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub access_token: Token,
}

#[derive(Debug)]
pub struct AuthenticatedAdmin {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub access_token: Token,
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let access_token = try_outcome!(request.guard::<BearerToken>().await).token;

        let state = try_outcome!(request.guard::<&State<ApiState>>().await);
            
        let access_token_info = unwrap_or_return!(
            state
            .find_session(&access_token)
            .await
            .ok_or(Outcome::Forward(()))
        );

        let authenticated_user = AuthenticatedUser {
            session_id: access_token_info.session_id,
            user_id: access_token_info.user_id,
            access_token,
        };

        Outcome::Success(authenticated_user)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedAdmin {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = try_outcome!(request.guard::<&State<ApiState>>().await);
        let user = try_outcome!(request.guard::<AuthenticatedUser>().await);

        state.with_user_info(&user.user_id, |user_info| -> Outcome<Self, Self::Error> {
            if !user_info.admin {
                return Outcome::Forward(());
            }

            let authenticated_admin = AuthenticatedAdmin {
                session_id: user.session_id,
                user_id: user.user_id,
                access_token: user.access_token,
                username: user_info.username.clone(),
            };
    
            Outcome::Success(authenticated_admin)
        })
        .await
        .unwrap_or_else(|| Outcome::Forward(()))
    }
}