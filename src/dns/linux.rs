use super::DNSManagerTrait;
use std::fmt::format;
use std::fs;
use std::io::Error;
use std::process::Command;

pub struct ResolvectlDNSManager {
    interface: String,
    original_dns: Option<String>,
    original_search: Option<String>,
}

impl DNSManagerTrait for ResolvectlDNSManager {
    fn new() -> ResolvectlDNSManager {
        ResolvectlDNSManager {
            interface: String::new(),
            original_dns: None,
            original_search: None,
        }
    }

    fn set_dns(&mut self, dns_servers: Vec<&str>, dns_search: Vec<&str>) -> Result<(), Error> {
        if dns_servers.is_empty() || self.interface.is_empty() {
            return Ok(());
        }

        // Store current DNS settings before changing them
        let status = Command::new("resolvectl")
            .arg("status")
            .arg(&self.interface)
            .output()?;
        let output = String::from_utf8_lossy(&status.stdout);

        // Parse and store original DNS servers and search domains
        if self.original_dns.is_none() && self.original_search.is_none() {
            for line in output.lines() {
                if line.contains("DNS Servers:") {
                    self.original_dns =
                        Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.contains("DNS Domain:") {
                    self.original_search =
                        Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
                }
            }
        }

        // Set new DNS servers
        Command::new("resolvectl")
            .arg("dns")
            .arg(&self.interface)
            .args(dns_servers.clone())
            .status()?;

        // Set new search domains if provided
        if !dns_search.is_empty() {
            Command::new("resolvectl")
                .arg("domain")
                .arg(&self.interface)
                .args(dns_search)
                .status()?;
        }

        log::debug!(
            "DNS set for interface {} with servers: {}",
            self.interface,
            dns_servers.join(",")
        );

        Ok(())
    }

    fn restore_dns(&self) -> Result<(), Error> {
        if self.interface.is_empty() {
            return Ok(());
        }

        // Restore original DNS servers if they were saved
        if let Some(dns) = &self.original_dns {
            if !dns.is_empty() {
                Command::new("resolvectl")
                    .arg("dns")
                    .arg(&self.interface)
                    .args(dns.split_whitespace())
                    .status()?;
            }
        }

        // Restore original search domains if they were saved
        if let Some(search) = &self.original_search {
            if !search.is_empty() {
                Command::new("resolvectl")
                    .arg("domain")
                    .arg(&self.interface)
                    .args(search.split_whitespace())
                    .status()?;
            }
        }

        log::debug!("DNS settings restored for interface {}", self.interface);
        Ok(())
    }

    fn with_interface(&mut self, interface: String) {
        self.interface = interface;
    }
}

pub struct ResolvConfDNSManager {
    restore: Option<String>,

    use_bind: bool,
}

impl DNSManagerTrait for ResolvConfDNSManager {
    fn new() -> ResolvConfDNSManager {
        ResolvConfDNSManager::new1(false)
    }

    fn set_dns(&mut self, dns_servers: Vec<&str>, dns_search: Vec<&str>) -> Result<(), Error> {
        if dns_servers.is_empty() {
            return Ok(());
        }

        // Save current resolv.conf content
        if self.restore.is_none() {
            self.restore = Some(fs::read_to_string("/etc/resolv.conf")?);
        }

        let mut resolv_conf = String::new();
        for server in dns_servers {
            resolv_conf.push_str(&format!("nameserver {}\n", server));
        }
        for search in dns_search {
            resolv_conf.push_str(&format!("search {}\n", search));
        }

        if self.use_bind {
            // create temp dir
            fs::create_dir_all("/run/corplink");
            fs::write("/run/corplink/resolv.conf", resolv_conf)?;
            // mount with bind
            Command::new("mount")
                .args(["--bind", "/run/corplink/resolv.conf", "/etc/resolv.conf"])
                .status()?;
        } else {
            fs::write("/etc/resolv.conf", resolv_conf)?;
        }

        Ok(())
    }

    fn restore_dns(&self) -> Result<(), Error> {
        if self.restore.is_none() {
            return Ok(());
        }

        if self.use_bind {
            // unmount resolv.conf
            Command::new("umount").arg("/etc/resolv.conf").status()?;
            // remove temp file
            fs::remove_file("/run/corplink/resolv.conf")?;
        } else {
            fs::write("/etc/resolv.conf", self.restore.as_ref().unwrap())?;
        }

        log::debug!("DNS: resolv.conf restored");
        Ok(())
    }
}

impl ResolvConfDNSManager {
    pub fn new1(use_bind: bool) -> ResolvConfDNSManager {
        ResolvConfDNSManager {
            restore: None,
            use_bind,
        }
    }
}
