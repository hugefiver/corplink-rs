use serde::{Deserialize, Serialize};
use std::{any::Any, cell::RefCell, io::Error, sync::Arc};

#[cfg(target_os = "macos")]
mod darwin;
#[cfg(target_os = "macos")]
pub use darwin::DNSManager;

#[cfg(target_os = "linux")]
mod linux;
// #[cfg(target_os = "linux")]
// pub use linux::{ResolvectlDNSManager, ResolvConfDNSManager};

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::DNSManager;

pub trait DNSManagerTrait: Any {
    fn new() -> Self
    where
        Self: Sized;
    fn set_dns(&mut self, dns_servers: Vec<&str>, dns_search: Vec<&str>) -> Result<(), Error>;
    fn restore_dns(&self) -> Result<(), Error>;

    fn with_interface(&mut self, interface: String) {}
}

pub struct NopDNSManager;

impl DNSManagerTrait for NopDNSManager {
    fn new() -> Self {
        NopDNSManager
    }

    fn set_dns(&mut self, _dns_servers: Vec<&str>, _dns_search: Vec<&str>) -> Result<(), Error> {
        Ok(())
    }

    fn restore_dns(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum VPNDnsMode {
    #[serde(alias = "off")]
    #[default]
    Off,

    #[serde(alias = "on")]
    Auto,

    #[cfg(target_os = "linux")]
    #[serde(alias = "resolvconf")]
    ResolvConf,

    #[cfg(target_os = "linux")]
    #[serde(alias = "resolvconf_bind", alias = "resolvconfbind")]
    ResolvConfBind,

    #[cfg(target_os = "linux")]
    #[serde(alias = "resolvctl")]
    Resolvectl,
    // TODO: Implement this for systemd-resolved
    //
    // #[cfg(target_os = "linux")]
    // #[serde(alias = "systemd-resolved")]
    // SystemdResolved,
}

impl VPNDnsMode {
    pub fn get_dns_manager(&self) -> Arc<RefCell<dyn DNSManagerTrait>> {
        match self {
            VPNDnsMode::Off => Arc::new(RefCell::new(NopDNSManager::new())),
            #[cfg(not(target_os = "linux"))]
            VPNDnsMode::Auto => Arc::new(RefCell::new(DNSManager::new())),
            #[cfg(target_os = "linux")]
            VPNDnsMode::Auto | VPNDnsMode::ResolvConfBind => {
                Arc::new(RefCell::new(linux::ResolvConfDNSManager::new1(true)))
            }
            #[cfg(target_os = "linux")]
            VPNDnsMode::ResolvConf => {
                Arc::new(RefCell::new(linux::ResolvConfDNSManager::new1(false)))
            }
            #[cfg(target_os = "linux")]
            VPNDnsMode::Resolvectl => Arc::new(RefCell::new(linux::ResolvectlDNSManager::new())),
            // #[cfg(target_os = "linux")]
            // VPNDnsMode::SystemdResolved => Arc::new(RefCell::new(linux::SystemdResolvedDNSManager::new())),
        }
    }

    pub fn is_off(&self) -> bool {
        matches!(self, VPNDnsMode::Off)
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, VPNDnsMode::Auto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpn_dns_mode_default() {
        let mode = VPNDnsMode::default();
        assert!(matches!(mode, VPNDnsMode::Off));
    }
}
