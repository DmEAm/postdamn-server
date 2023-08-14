use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::result::{DatabaseErrorKind, Error};

pub enum PostdamnError {
    DatabaseError(Error),
}

impl IntoResponse for PostdamnError {
    fn into_response(self) -> Response {
        match self {
            PostdamnError::DatabaseError(Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation | DatabaseErrorKind::ForeignKeyViolation,
                message,
            )) => problemdetails::new(StatusCode::BAD_REQUEST)
                .with_title("One or more errors occurred.")
                .with_detail(message.details().unwrap_or(message.message())),
            _ => problemdetails::new(StatusCode::INTERNAL_SERVER_ERROR)
                .with_title("Internal server error."),
        }
        .into_response()
    }
}

impl From<Error> for PostdamnError {
    fn from(inner: Error) -> Self {
        PostdamnError::DatabaseError(inner)
    }
}
