use serde::{Deserialize, Serialize};

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserOpt {
    pub id: Option<i64>,
    pub username: Option<String>,
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
