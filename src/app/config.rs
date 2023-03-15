use std::{collections::HashSet, net::IpAddr, str::FromStr};

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfiguration {
    pub ps_realms: Vec<String>,
    pub ps_allowed_ips: Vec<IpAddr>,
    pub ps_allowed_sids: HashSet<i64>,
    pub ps_blocked_sids: HashSet<i64>,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        AppConfiguration {
            ps_realms: Vec::new(),
            ps_allowed_ips: vec![IpAddr::from_str("127.0.0.1").unwrap()],
            ps_allowed_sids: HashSet::new(),
            ps_blocked_sids: HashSet::new(),
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
