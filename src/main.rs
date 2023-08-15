extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate utoipa;

use std::{default::Default, env, net::SocketAddr};

use axum::{
    async_trait,
    extract::FromRequest,
    extract::State,
    handler::Handler,
    http::{Request, StatusCode},
    Json, RequestExt,
    response::{IntoResponse, Response},
    Router,
    routing::{get, post},
};
use diesel::{
    Connection, ConnectionResult, PgConnection,
    result::{DatabaseErrorKind, Error},
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use dotenvy::dotenv;
use problemdetails::Problem;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    IntoParams, Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    ToSchema,
};
use utoipa_swagger_ui::SwaggerUi;

use postdamn::{
    core::ValidatedJson,
    error::PostdamnError,
    models::{Role, User},
    services::Example,
};

pub mod params;
pub mod schema;

pub async fn establish_connection() -> ConnectionResult<AsyncPgConnection> {
    let url = env::var("DATABASE_URL").expect("Database url env var not set");
    Ok(AsyncPgConnection::establish(&url).await?)
}
mod users {
    use axum::{extract::Query, http::StatusCode, Json};
    use axum::body::HttpBody;
    use diesel::{
        debug_query, insert_into,
        pg::Pg,
        prelude::*,
        query_builder::AsQuery,
        result::{DatabaseErrorKind, Error},
    };
    use diesel_async::RunQueryDsl;
    use problemdetails::Problem;
    use serde::{Deserialize, Serialize};
    use validator::{Validate, ValidationError};

    use schema::security::users;

    use crate::{establish_connection, params, PostdamnError, schema, User, ValidatedJson};

    #[derive(Debug)]
    pub struct GetUsersRequest {
        page: params::Page,
        search: params::Search,
    }
    /// All users info
    #[utoipa::path(get, path = "/api/v1/users", responses(
    (status = StatusCode::OK, description = "List of users", body = [User])),
    params(params::Page, params::Search)
    )]
    pub async fn get_users_list(
        page: Query<params::Page>,
        search: Query<params::Search>,
    ) -> Result<(StatusCode, Json<Vec<User>>), PostdamnError> {
        use crate::schema::security::users::dsl::*;
        let mut connection = establish_connection().await.map_err(|e| e.into())?;
        tracing::debug!("processing request");

        let mut query = users
            .order_by(name)
            .then_order_by(id)
            .offset(page.offset.unwrap_or_default())
            .limit(page.limit.unwrap_or_default())
            .into_boxed();

        if let Some(q) = search.q.as_ref() {
            query = query.filter(name.like(format!("%{}%", q)))
        }
        tracing::trace!("Execute SQL {}", debug_query::<Pg, _>(&query));
        let u: Vec<User> = query.load(&mut connection).await.map_err(|e| e.into())?;
        Ok((StatusCode::OK, Json(u)))
    }

    #[derive(Deserialize, ToSchema, Validate, Insertable)]
    #[diesel(table_name = users)]
    pub struct CreateUser {
        name: String,
        #[validate(email)]
        email: String,
        #[validate(phone)]
        phone: String,
    }

    /// Create user
    #[utoipa::path(post, path = "/api/v1/users", responses(
    (status = StatusCode::CREATED, description = "User successfully created", body = User)),
    params()
    )]
    pub async fn post_user(
        ValidatedJson(payload): ValidatedJson<CreateUser>,
    ) -> Result<(StatusCode, Json<User>), PostdamnError> {
        use crate::schema::security::users::dsl::*;
        let mut connection = establish_connection().await.map_err(|e| e.into())?;
        let result = insert_into(users)
            .values(payload)
            .get_result::<User>(&mut connection)
            .await
            .map_err(|e| e.into())?;
        Ok((StatusCode::CREATED, Json(result)))
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
    use axum::error_handling::HandleErrorLayer;

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
