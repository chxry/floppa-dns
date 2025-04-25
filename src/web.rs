use askama::Template;
use axum::response::{Html, Redirect};

#[derive(Template)]
#[template(path = "home.html")]
struct Home {}

pub async fn home() -> Html<String> {
  Html(Home {}.render().unwrap())
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login {}

pub async fn login() -> Html<String> {
  Html(Login {}.render().unwrap())
}

pub async fn notfound() -> Redirect {
  Redirect::to("/")
}
