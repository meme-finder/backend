use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "MEILI_URL", default = "http://localhost:7700")]
    pub meili_url: String,

    #[envconfig(from = "MEILI_MASTER_KEY", default = "key")]
    pub meili_key: String,
}

pub fn get_config() -> Config {
    Config::init_from_env().expect("Can't load config")
}
