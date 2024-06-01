#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub fingerprint_image: Vec<u8>,
}

#[derive(Debug)]
pub struct Credential {
    pub id: i64,
    pub user_id: i64,
    pub site: String,
    pub site_username: String,
    pub site_password: String,
}