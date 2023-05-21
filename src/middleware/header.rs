use axum::{
    http::{HeaderValue, Request},
    middleware::Next,
    response::IntoResponse,
};
use std::collections::HashMap;
use std::fmt::Debug;

pub async fn no_cache_header<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse
where
    B: Debug,
{
    // handler request
    let mut response = next.run(req).await;

    let mut m = HashMap::new();
    m.insert("Cache-Control", "no-cache,must-revalidate,no-store");
    m.insert("Pragma", "no-cache");
    m.insert("Expires", "-1");

    // set no cache header
    for (key, value) in m {
        response
            .headers_mut()
            .insert(key, HeaderValue::from_str(value).unwrap());
    }

    response
}
