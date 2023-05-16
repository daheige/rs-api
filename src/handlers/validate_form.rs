use async_trait::async_trait;
use axum::extract::{rejection::FormRejection, Form, FromRequest};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NameInput {
    #[validate(length(min = 1, message = "can not be empty"))]
    pub name: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

/// impl FromRequest trait
/// these bounds are required by `async_trait`
#[async_trait]
impl<S, B, T> FromRequest<S, B> for ValidatedForm<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Form<T>: FromRequest<S, B, Rejection = FormRejection>,
{
    type Rejection = ServerError;
    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
}

/// convert the error to response
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(err) => {
                // let message = format!("input validation error: [{}]", self).replace('\n', ", ");
                let msg = format!("input validation error: [{}]", err).replace('\n', ", ");
                (
                    StatusCode::OK,
                    Json(super::Reply {
                        code: 1001,
                        message: msg,
                        data: Some(super::EmptyObject {}),
                    }),
                )
            }
            ServerError::AxumFormRejection(_) => (
                StatusCode::BAD_REQUEST,
                Json(super::Reply {
                    code: 500,
                    message: format!("param error:{}", self.to_string()),
                    data: Some(super::EmptyObject {}),
                }),
            ),
        }
        .into_response()
    }
}
