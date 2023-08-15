use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::ConnectionError;
use diesel::result::{DatabaseErrorKind, Error};

pub enum PostdamnError {
    DatabaseError(Error),
    ConnectionError(ConnectionError),
}

impl IntoResponse for PostdamnError {
    fn into_response(self) -> Response {
        const ERROR_TITLE: &str = "One or more errors occurred.";
        match self {
            PostdamnError::DatabaseError(Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation | DatabaseErrorKind::ForeignKeyViolation,
                message,
            )) => problemdetails::new(StatusCode::BAD_REQUEST)
                .with_title(ERROR_TITLE)
                .with_detail(message.details().unwrap_or(message.message())),
            PostdamnError::ConnectionError(error) => {
                problemdetails::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_title(ERROR_TITLE)
                    .with_detail(error.to_string())
            }
            _ => problemdetails::new(StatusCode::INTERNAL_SERVER_ERROR)
                .with_title("Internal server error."),
        }
        .into_response()
    }
}

impl Into<PostdamnError> for ConnectionError {
    fn into(self) -> PostdamnError {
        PostdamnError::ConnectionError(self)
    }
}
impl Into<PostdamnError> for Error {
    fn into(self) -> PostdamnError {
        PostdamnError::DatabaseError(self)
    }
}
