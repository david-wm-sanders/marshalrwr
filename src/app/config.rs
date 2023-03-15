use std::{
    net::IpAddr,
    str::FromStr,
};

use serde::{Serialize, Deserialize};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfiguration {
    pub ps_realms: Vec<String>,
    pub ps_allowed_ips: Vec<IpAddr>,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        AppConfiguration {
            ps_realms: Vec::new(),
            ps_allowed_ips: vec![IpAddr::from_str("127.0.0.1").unwrap()],
        }
    }
}

impl AppConfiguration {
    pub fn build() -> Result<Self, figment::Error> {
        let app_config: AppConfiguration =
            Figment::from(Serialized::defaults(AppConfiguration::default()))
                .merge(Toml::file("marshalrwr.toml"))
                .merge(Env::prefixed("MRWR_"))
                .extract()?;
        Ok(app_config)
    }
}