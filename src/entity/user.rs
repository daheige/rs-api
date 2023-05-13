use serde::{Deserialize, Serialize};

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
}

// form data
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserForm {
    #[serde(default)]
    name: String,

    // The default value is ""
    // if this parameter is not specified, this parameter is mandatory
    #[serde(default)]
    email: String,

    #[serde(default)]
    age: i32,
}
