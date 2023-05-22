use serde::{Deserialize, Serialize};

pub mod index;
pub mod json_or_form;
pub mod validate_form;

#[derive(Deserialize, Serialize, Debug)]
pub struct Reply<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

// empty object,like {}
#[derive(Deserialize, Serialize, Debug)]
pub struct EmptyObject {}

// empty array,like:[]
type EmptyArray = Vec<EmptyObject>;
