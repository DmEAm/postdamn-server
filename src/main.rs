extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate utoipa;

use std::{default::Default, env, net::SocketAddr};

use axum::{
    extract::State,
    handler::Handler,
    http::StatusCode,
    Json,
    response::IntoResponse,
    Router,
    routing::{get, post},
};
use diesel::{Connection, debug_query, pg::Pg, PgConnection, prelude::*};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    IntoParams, Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    ToSchema,
};

use postdamn::{
    models::{Role, User},
    services::Example,
};
use utoipa_swagger_ui::SwaggerUi;

pub mod models;
pub mod params;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    let url = env::var("DATABASE_URL").expect("Database url env var not set");
    PgConnection::establish(&url).ok().unwrap()
}

mod users {
    use axum::{http::StatusCode, Json, extract::Query};
    use diesel::{prelude::*};
    use diesel::query_builder::AsQuery;
    use serde::{Deserialize, Serialize};

    use crate::{debug_query, establish_connection, params, Pg};
    use crate::models::User;

    #[derive(Debug)]
    pub struct GetUsersRequest {
        page: params::Page,
        search: params::Search,
    }
    /// All users info
    #[utoipa::path(get, path = "/api/v1/users", responses(
    (status = 200, description = "List of users", body = [User])),
    params(params::Page, params::Search)
    )]
    pub async fn get_users_list(
        page: Query<params::Page>,
        search: Query<params::Search>,
    ) -> (StatusCode, Json<Vec<User>>) {
        use crate::schema::security::users::dsl::*;
        let mut connection = establish_connection();
        tracing::debug!("processing request");

        let mut query = users
            .order_by(name)
            .then_order_by(id)
            .offset(page.offset.unwrap_or_default())
            .limit(page.limit.unwrap_or_default())
            .into_boxed();

        if let Some(q) = search.q.as_ref() {
            query = query.filter(name.eq(q))
        }
        tracing::trace!("Execute SQL {}", debug_query::<Pg, _>(&query));
        let u: Vec<User> = query
            .load::<User>(&mut connection)
            .expect("Error loading users");
        (StatusCode::OK, Json(u))
    }

    #[derive(Deserialize, ToSchema)]
    pub struct CreateUser {
        name: String,
    }

    /// Create user
    #[utoipa::path(post, path = "/api/v1/users", responses(
    (status = 201, description = "User successfully created", body = User)),
    params()
    )]
    pub async fn post_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
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
}
#[derive(OpenApi)]
#[openapi(
info(
    description = "Postdamn OAuth security provider",
    title = "Postdamn Security",
    version = "v1"
),
tags(
(name = "users", description = "Users security permissions management")
),
components(
schemas(User, users::CreateUser)
),
paths(
users::get_users_list,
users::post_user,
),
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "postdamn=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let user_routes = Router::new().route("/", get(users::get_users_list).post(users::post_user));
    let api_routes = Router::new().nest("/users", user_routes);
    let app = Router::new()
        .nest("/api/:version", api_routes)
        .merge(SwaggerUi::new("/swagger").url("/swagger/v1/swagger.json", ApiDoc::openapi()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
