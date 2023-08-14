extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate utoipa;

use std::{default::Default, env, net::SocketAddr};
use std::error::Error;

use axum::{
    async_trait,
    extract::FromRequest,
    extract::State,
    handler::Handler,
    http::{Request, StatusCode},
    Json, RequestExt,
    response::IntoResponse,
    Router,
    routing::{get, post},
};
use axum::BoxError;
use diesel::{Connection, PgConnection};
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
    models::{Role, User},
    services::Example,
};

pub mod params;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    let url = env::var("DATABASE_URL").expect("Database url env var not set");
    PgConnection::establish(&url).ok().unwrap()
}

mod users {
    use axum::{extract::Query, http::StatusCode, Json};
    use diesel::{
        debug_query, insert_into,
        pg::Pg,
        prelude::*,
        result::{DatabaseErrorKind, Error},
    };
    use diesel::query_builder::AsQuery;
    use problemdetails::Problem;
    use serde::{Deserialize, Serialize};
    use validator::{Validate, ValidationError};

    use schema::security::users;

    use crate::{establish_connection, params, schema, User, ValidatedJson};

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
            query = query.filter(name.like(format!("%{}%", q)))
        }
        tracing::trace!("Execute SQL {}", debug_query::<Pg, _>(&query));
        let u: Vec<User> = query
            .load::<User>(&mut connection)
            .expect("Error loading users");
        (StatusCode::OK, Json(u))
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
    ) -> Result<(StatusCode, Json<User>), Problem> {
        use crate::schema::security::users::dsl::*;
        let mut connection = establish_connection();
        let result = insert_into(users)
            .values(payload)
            .get_result::<User>(&mut connection)
            .map_err(|e| match e {
                Error::DatabaseError(
                    DatabaseErrorKind::UniqueViolation | DatabaseErrorKind::ForeignKeyViolation,
                    message,
                ) => problemdetails::new(StatusCode::BAD_REQUEST)
                    .with_title("One or more errors occurred.")
                    .with_detail(message.details().unwrap_or(message.message())),
                _ => problemdetails::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_title("Internal server error."),
            })?;
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
