use rocket::{
    Rocket, Build, get, post, routes, State, uri,
    response::Redirect,
    response::status,
};

use state::{
    AuthenticatedAdmin, ApiState,
    ui::UserInfo
};

pub fn update_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
    .mount("/admin", routes![
        index_unauthorized,
        index,
        login
    ])
}

use askama::Template;



#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    pub current_user: String,
    pub users: Vec<UserInfo>,
}

#[get("/", rank = 1)]
async fn index(
    state: &State<ApiState>,
    user: AuthenticatedAdmin,
) -> IndexTemplate {

    let users = state.ui_get_all_users().await.unwrap_or_else(|| vec![]);

    IndexTemplate { 
        current_user: user.username.clone(),
        users,
    }
}

#[derive(Template)]
#[template(path = "login_form.html")]
struct LoginFromTemplate {}

#[get("/", rank = 2)]
async fn index_unauthorized(
) -> LoginFromTemplate {
    LoginFromTemplate {}
}

#[post("/login")]
async fn login(
) -> Result<Redirect, status::Forbidden<()>> {
    Ok(Redirect::to(uri!("/admin")))
}