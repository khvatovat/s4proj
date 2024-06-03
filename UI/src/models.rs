use druid::{Data, Lens};
#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub fingerprint_image: Vec<u8>,
}

#[derive(Debug, Clone, Data, Lens, PartialEq)]
pub struct Credential {
    pub id: i32,
    pub username: String,
    pub site: String,
    pub site_username: String,
    pub site_password: String,
}