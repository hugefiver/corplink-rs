use serde_with::{serde_as, DefaultOnNull};
use std::fmt;
use std::ops::Deref;
use tokio::fs;

use serde::{Deserialize, Serialize};

use crate::dns::VPNDnsMode;
use crate::state::State;
use crate::utils;

const DEFAULT_DEVICE_NAME: &str = "DollarOS";
const DEFAULT_INTERFACE_NAME: &str = "corplink";

pub const PLATFORM_LDAP: &str = "ldap";
pub const PLATFORM_CORPLINK: &str = "feilian";
pub const PLATFORM_OIDC: &str = "OIDC";
// aka feishu
pub const PLATFORM_LARK: &str = "lark";
#[allow(dead_code)]
pub const PLATFORM_WEIXIN: &str = "weixin";
// aka dingding
#[allow(dead_code)]
pub const PLATFORM_DING_TALK: &str = "dingtalk";
// unknown
#[allow(dead_code)]
pub const PLATFORM_AAD: &str = "aad";

pub const STRATEGY_LATENCY: &str = "latency";
pub const STRATEGY_DEFAULT: &str = "default";

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub company_name: String,
    pub username: String,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub code: Option<String>,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub server: Option<String>,
    pub interface_name: Option<String>,
    pub debug_wg: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_requests: Option<BoolOr<String>>,
    #[serde(skip_serializing)]
    pub conf_file: Option<String>,
    pub state: Option<State>,
    pub vpn_server_name: Option<String>,
    pub vpn_select_strategy: Option<String>,

    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(default)]
    pub use_vpn_dns: VPNDnsMode,

    #[serde(default)]
    pub routing: RouteSetting,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum BoolOr<T> {
    Bool(bool),
    Value(T),
}

impl<T> Default for BoolOr<T> {
    fn default() -> Self {
        BoolOr::Bool(false)
    }
}

impl<T> BoolOr<T> {
    pub fn as_bool(&self) -> bool {
        matches!(self, BoolOr::Bool(true)) || matches!(self, BoolOr::Value(_))
    }

    pub fn is_true(&self) -> bool {
        self.as_bool()
    }

    pub fn is_false(&self) -> bool {
        !self.as_bool()
    }

    pub fn get_value(&self) -> Option<&T> {
        match self {
            BoolOr::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_value_with_default<'a>(&'a self, default: &'a T) -> &'a T {
        match self {
            BoolOr::Value(v) => v,
            BoolOr::Bool(true) => default,
            _ => default,
        }
    }

    pub fn map<E>(self, f: impl FnOnce(T) -> E) -> BoolOr<E> {
        match self {
            BoolOr::Bool(b) => BoolOr::Bool(b),
            BoolOr::Value(v) => BoolOr::Value(f(v)),
        }
    }

    pub fn as_ref(&self) -> BoolOr<&T> {
        match self {
            BoolOr::Bool(b) => BoolOr::Bool(*b),
            BoolOr::Value(v) => BoolOr::Value(v),
        }
    }

    pub fn as_deref(&self) -> BoolOr<&T::Target>
    where
        T: Deref,
    {
        self.as_ref().map(|t| t.deref())
    }
}

impl<T: PartialEq + Default> BoolOr<T> {
    pub fn get_value_no_zero(&self) -> Option<&T> {
        match self {
            BoolOr::Value(v) if *v != T::default() => Some(v),
            _ => None,
        }
    }

    pub fn get_value_no_zero_with_default(self, default: T) -> Option<T> {
        match self {
            BoolOr::Value(v) if v != T::default() => Some(v),
            BoolOr::Bool(true) => Some(default),
            _ => None,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = serde_json::to_string_pretty(self).unwrap();
        write!(f, "{s}")
    }
}

impl Config {
    pub async fn from_file(file: &str) -> Config {
        let conf_str = fs::read_to_string(file)
            .await
            .unwrap_or_else(|e| panic!("failed to read config file {file}: {e}"));

        let mut conf: Config = serde_json::from_str(&conf_str[..])
            .unwrap_or_else(|e| panic!("failed to parse config file {file}: {e}"));

        conf.conf_file = Some(file.to_string());
        let mut update_conf = false;
        if conf.interface_name.is_none() {
            conf.interface_name = Some(DEFAULT_INTERFACE_NAME.to_string());
            update_conf = true;
        }
        if conf.device_name.is_none() {
            conf.device_name = Some(DEFAULT_DEVICE_NAME.to_string());
            update_conf = true;
        }
        if conf.device_id.is_none() {
            conf.device_id = Some(format!(
                "{:x}",
                md5::compute(conf.device_name.clone().unwrap())
            ));
            update_conf = true;
        }
        match &conf.private_key {
            Some(private_key) => match conf.public_key {
                Some(_) => {
                    // both keys exist, do nothing
                }
                None => {
                    // only private key exists, generate public from private
                    let public_key = utils::gen_public_key_from_private(private_key).unwrap();
                    conf.public_key = Some(public_key);
                    update_conf = true;
                }
            },
            None => {
                // no key exists, generate new
                let (public_key, private_key) = utils::gen_wg_keypair();
                (conf.public_key, conf.private_key) = (Some(public_key), Some(private_key));
                update_conf = true;
            }
        }
        if update_conf {
            conf.save().await;
        }
        conf
    }

    pub async fn save(&self) {
        let file = self.conf_file.as_ref().unwrap();
        let data = format!("{}", &self);
        fs::write(file, data).await.unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RouteSetting {
    #[serde(default)]
    pub mode: RoutingMode,

    // only for `Split` mode
    #[serde(default)]
    pub include_dynamic_domain_route_split: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub enum RoutingMode {
    #[serde(alias = "split")]
    #[default]
    Split,

    #[serde(alias = "full")]
    Full,
}

#[derive(Serialize, Clone)]
pub struct WgConf {
    // standard wg conf
    pub address: String,
    pub address6: String,
    pub peer_address: String,
    pub mtu: u32,
    pub public_key: String,
    pub private_key: String,
    pub peer_key: String,
    pub route: Vec<String>,

    // extent confs
    pub dns: String,

    // corplink confs
    pub protocol: i32,
}
