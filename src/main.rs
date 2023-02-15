#[macro_use]
extern crate diesel;
extern crate chrono;

use diesel::prelude::*;

pub mod schema;
pub mod models;

use std::default::Default;
use diesel::{
    Connection, PgConnection,
    pg::Pg,
    debug_query,
};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::{
    extract::State,
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use axum::handler::Handler;
use tokio_postgres::NoTls;
use postdamn::{
    models::{User, Role},
    services::{Example},
};

pub fn establish_connection() -> PgConnection {
    let url = env::var("DATABASE_URL").expect("Database url env var not set");
    PgConnection::establish(&url).ok().unwrap()
}

async fn get_users_list() -> (StatusCode, Json<Vec<User>>) {
    use self::schema::security::users::dsl::*;
    let mut connection = establish_connection();
    let q = users
        .filter(name.eq("test"))
        .order_by(name)
        .then_order_by(id)
        .limit(5);
    tracing::debug!("Run SQL {}", debug_query::<Pg, _>(&q));
    let u: Vec<User> = q.load::<User>(& mut connection).expect("Error loading users");
    (StatusCode::OK, Json(u))
}

async fn post_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    let user = User {
        id: Default::default(),
        created_at: Default::default(),
        updated_at: None,
        name: payload.name,
        email: "".to_string(),
        phone: "".to_string(),
    };

    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let user_routes = Router::new()
        .route("/", get(get_users_list).post(post_user));
    let api_routes = Router::new()
        .nest("/users", user_routes);
    let app = Router::new()
        .nest("/api/:version", api_routes);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
