use serde::{Deserialize, Serialize};

pub mod index;
pub mod validate_form;

#[derive(Deserialize, Serialize, Debug)]
struct Reply<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

// empty object,like {}
#[derive(Deserialize, Serialize, Debug)]
pub struct EmptyObject {}

// empty array,like:[]
type EmptyArray = Vec<EmptyObject>;
