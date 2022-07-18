use rocket::{
    http::{
        Status,
        hyper::header::AUTHORIZATION
    },
    request::{Request, FromRequest, Outcome},
    State,
};
use crate::{
    unwrap_or_return,
    tokens::Token,
    state::{SessionId, UserId, ApiState},
};

#[derive(Debug)]
pub struct BearerToken {
    pub token: Token,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_string = unwrap_or_return!(
            request
            .headers()
            .get_one(AUTHORIZATION.as_str())
            .ok_or(Outcome::Failure((Status::Unauthorized, "Missing Authorization header")))
        );

        let (bearer, mut token_str) = unwrap_or_return!(
            auth_string
            .split_once(' ')
            .ok_or(Outcome::Failure((Status::Unauthorized, "Malformed Authorization header")))
        );

        if !bearer.eq_ignore_ascii_case("Bearer") {
            return Outcome::Failure((Status::Unauthorized, "Invalid Authorization type"));
        };

        token_str = token_str.trim();

        let token = unwrap_or_return!(
            Token::from_str(token_str)
            .map_err(|_| Outcome::Failure((Status::Unauthorized, "Malformed Authorization token")))
        );

        let bearer = Self { token };

        Outcome::Success(bearer)
    }
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub access_token: Token,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let access_token = unwrap_or_return!(
            request.local_cache_async(async {
                request.guard::<BearerToken>().await.succeeded().map(|bearer| bearer.token)
            })
            .await
            .ok_or(Outcome::Failure((Status::Unauthorized, ())))
        );

        let state = request.guard::<&State<ApiState>>().await.succeeded().unwrap();
            
        let access_token_info = unwrap_or_return!(
            state
            .find_session(&access_token)
            .await
            .ok_or(Outcome::Failure((Status::Unauthorized, ())))
        );
    
        let authenticated_user = AuthenticatedUser {
            session_id: access_token_info.session_id,
            user_id: access_token_info.user_id,
            access_token
        };

        Outcome::Success(authenticated_user)
    }
}