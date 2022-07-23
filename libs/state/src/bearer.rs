use std::marker::PhantomData;
use rocket::{
    request::{Request, FromRequest, Outcome},
    State, 
    outcome::try_outcome,
};
use utils::{unwrap_or_return, Token, IntoToken};
use crate::{
    SessionId, UserId, 
    state::ApiState,
};

#[derive(Debug, Clone)]
pub struct AuthenticatedUserInfo {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub access_token: Token,
}

#[derive(Debug)]
pub struct AuthenticatedUser<T> {
    pub info: AuthenticatedUserInfo,
    pub _ph: PhantomData<T>,
}

#[derive(Debug)]
pub struct AuthenticatedAdmin<T> {
    pub info: AuthenticatedUserInfo,
    pub username: String,
    pub _ph: PhantomData<T>,
}


#[rocket::async_trait]
impl<'r, T> FromRequest<'r> for AuthenticatedUser<T> where T: FromRequest<'r, Error = ()> + IntoToken + Send {
    type Error = T::Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let access_token = try_outcome!(request.guard::<T>().await).into_token();

        let state = try_outcome!(request.guard::<&State<ApiState>>().await);
            
        let access_token_info = unwrap_or_return!(
            state
            .find_session(&access_token)
            .await
            .ok_or(Outcome::Forward(()))
        );

        let authenticated_user = AuthenticatedUser {
            info: AuthenticatedUserInfo {
                session_id: access_token_info.session_id,
                user_id: access_token_info.user_id,
                access_token,
            },
            _ph: PhantomData,
        };

        Outcome::Success(authenticated_user)
    }
}

#[rocket::async_trait]
impl<'r, T> FromRequest<'r> for AuthenticatedAdmin<T> where T: FromRequest<'r, Error = ()> + IntoToken + Send {
    type Error = T::Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = try_outcome!(request.guard::<&State<ApiState>>().await);
        let user = try_outcome!(request.guard::<AuthenticatedUser<T>>().await);

        state.with_user_info(&user.info.user_id, |user_info| -> Outcome<Self, Self::Error> {
            if !user_info.admin {
                return Outcome::Forward(());
            }

            let authenticated_admin = AuthenticatedAdmin {
                info: user.info.clone(),
                username: user_info.username.clone(),
                _ph: PhantomData,
            };
    
            Outcome::Success(authenticated_admin)
        })
        .await
        .unwrap_or_else(|| Outcome::Forward(()))
    }
}