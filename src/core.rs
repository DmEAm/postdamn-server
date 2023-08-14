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
use validator::{Validate, ValidationErrors};

pub struct ValidatedJson<J>(pub J);

#[async_trait]
impl<S, B, J> FromRequest<S, B> for ValidatedJson<J>
where
    B: Send + 'static,
    S: Send + Sync,
    J: Validate + 'static,
    Json<J>: FromRequest<(), B>,
{
    type Rejection = Problem;

    async fn from_request(req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = req.extract::<Json<J>, _>().await.map_err(|e| {
            problemdetails::new(StatusCode::BAD_REQUEST)
                .with_title("Invalid json.")
                .with_detail("Cannot deserialize json.")
        })?;
        data.validate().map_err(|e| {
            problemdetails::new(StatusCode::BAD_REQUEST)
                .with_title("One ore more validation errors occurred.")
                .with_detail(e.to_string())
        })?;
        Ok(Self(data))
    }
}
