use axum::{
    http::{
        header::{self, AsHeaderName},
        HeaderMap, HeaderValue, Request,
    },
    middleware::Next,
    response::IntoResponse,
};
use std::fmt::Debug;
use std::ops::Sub;
use std::time::Instant;
use uuid::Uuid;

pub async fn access_log<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse
where
    B: Debug,
{
    let method = req.method();
    let body = req.body();
    let uri = req.uri();
    let path = uri.path();
    let query = uri.query();
    let headers = req.headers();
    let ua = get_header(headers, header::USER_AGENT);
    let mut request_id = get_header(headers, "x-request-id");
    if request_id.is_empty() {
        request_id = Uuid::new_v4().to_string()
    }

    let start_time = Instant::now();

    // println!("request:{:?}", req);
    println!(
        "exec begin method:{} uri:{} path:{} request body:{:?} query:{:?} ua:{} request_id:{}",
        method, uri, path, body, query, ua, request_id,
    );

    // insert x-request-id into headers
    let (mut parts, body) = req.into_parts();
    // parts
    //     .headers
    //     .insert("x-request-id", request_id.parse().unwrap());

    parts.headers.insert(
        "x-request-id",
        HeaderValue::from_str(request_id.as_str()).unwrap(),
    );

    // change request with new parts and body
    let req = Request::from_parts(parts, body);

    // handler request
    let response = next.run(req).await;

    // exec request end
    let end_time = Instant::now();
    println!(
        "exec end,request_id:{},exec_time:{}ms",
        request_id,
        end_time.sub(start_time).as_millis(),
    );

    response
}

fn get_header<K: AsHeaderName>(headers: &HeaderMap, key: K) -> String {
    let s = headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or("".to_string());
    s
}
