use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;

pub(crate) struct Adapter {
    name: String,
    ipv4_address: Option<Ipv4Addr>,
    subnet_mask: Option<Ipv4Addr>,
}

impl Adapter {
    pub(crate) fn broadcast_address(&mut self) -> Option<IpAddr> {
        if self.ipv4_address.is_none() || self.subnet_mask.is_none() {
            return None;
        }

        let ip_address = self.ipv4_address.unwrap().octets();
        let subnet_mask = self.subnet_mask.unwrap().octets();

        let mut broadcast_address = [0; 4];
        for i in 0..4 {
            broadcast_address[i] = ip_address[i] | !subnet_mask[i];
        }

        Some(IpAddr::from(broadcast_address))
    }
    pub(crate) fn get_adapter_name(&self) -> String {
        self.name.clone()
    }
}

pub(crate) fn get_adapters() -> Vec<Adapter> {
    // Run ipconfig
    let output = Command::new("ipconfig")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8_lossy(&output.stdout);

    let mut adapters = Vec::new();
    let mut current_adapter: Option<Adapter> = None;

    for line in output_str.lines() {
        // Check if the line contains the name of the Adapter
        if line.ends_with(":") {
            // If there is an adapter in the current_adapter variable, push it to the adapters vector
            if let Some(adapter) = current_adapter {
                if !adapter.ipv4_address.is_none() {
                    adapters.push(adapter);
                }
            }

            // Create a new adapter
            current_adapter = Some(Adapter {
                name: line.trim_end_matches(":").to_string(),
                ipv4_address: None,
                subnet_mask: None,
            });
        }
        if line.contains("IPv4 Address") {
            let ip_address = line.split(":").nth(1).unwrap().trim();
            let ip_address = ip_address.split("(").nth(0).unwrap().trim();
            match ip_address.parse::<Ipv4Addr>() {
                Ok(ip) => current_adapter.as_mut().unwrap().ipv4_address = Some(ip),
                Err(_) => {}
            }
        }
        if line.contains("Subnet Mask") {
            let ip_address = line.split(":").nth(1).unwrap().trim();
            let ip_address = ip_address.split("(").nth(0).unwrap().trim();
            match ip_address.parse::<Ipv4Addr>() {
                Ok(ip) => current_adapter.as_mut().unwrap().subnet_mask = Some(ip),
                Err(_) => {}
            }
        }
    }

    if let Some(adapter) = current_adapter {
        if !adapter.ipv4_address.is_none() {
            adapters.push(adapter);
        }
    }
    adapters
}
