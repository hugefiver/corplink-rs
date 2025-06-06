#[derive(serde::Deserialize, Debug)]
pub struct Resp<T> {
    pub code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespCompany {
    pub name: String,
    pub zh_name: String,
    pub en_name: String,
    pub domain: String,
    pub enable_self_signed: bool,
    pub self_signed_cert: String,
    pub enable_public_key: bool,
    pub public_key: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespLoginMethod {
    pub login_enable_ldap: bool,
    pub login_enable: bool,
    pub login_orders: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespTpsLoginMethod {
    pub alias: String,
    pub login_url: String,
    pub token: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespCorplinkLoginMethod {
    pub mfa: bool,
    pub auth: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespLogin {
    #[serde(default)]
    pub url: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespOtp {
    pub url: String,
    pub code: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespVpnInfo {
    pub api_port: u16,
    pub vpn_port: u16,
    pub ip: String,
    // 1 for tcp, 2 for udp, we only support udp for now
    pub protocol_mode: i32,
    // useless
    pub name: String,
    pub en_name: String,
    pub icon: String,
    pub id: i32,
    pub timeout: i32,
    pub exclude: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespWgExtraInfo {
    pub vpn_mtu: u32,
    pub vpn_dns: String,
    pub vpn_dns_backup: String,

    pub vpn_route_full: Vec<String>,
    pub vpn_route_split: Vec<String>,
    pub v6_route_full: Vec<String>,
    pub v6_route_split: Vec<String>,

    pub vpn_dns_domain_split: Option<Vec<String>>,
    pub vpn_dynamic_domain_route_split: Option<Vec<(String, Vec<String>)>>,

    #[serde(default)]
    pub vpn_global_exclude: VPNGlobalExclude,
}


#[derive(serde::Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub struct VPNGlobalExclude {
    pub domains: Option<Vec<String>>,
    pub ips: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespWgInfo {
    pub ip: String,
    pub ipv6: String,
    pub ip_mask: String,
    pub public_key: String,
    pub setting: RespWgExtraInfo,
    pub mode: u32,
}
