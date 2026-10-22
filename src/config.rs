pub struct Config {
    pub database_url: String,
    pub server_address: String,
}

impl Config {
    pub fn from_env() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let server_address = std::env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");

        Config {
            database_url,
            server_address,
        }
    }
}
