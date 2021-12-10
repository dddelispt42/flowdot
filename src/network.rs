use dns_lookup::lookup_host;
use ipaddress::IPAddress;
use std::net::IpAddr;

fn is_valid_hostname(hostname: &str) -> bool {
    fn is_valid_char(byte: u8) -> bool {
        (byte >= b'a' && byte <= b'z')
            || (byte >= b'A' && byte <= b'Z')
            || (byte >= b'0' && byte <= b'9')
            || byte == b'-'
            || byte == b'.'
    }

    !(hostname.bytes().any(|byte| !is_valid_char(byte))
        || hostname.ends_with('-')
        || hostname.starts_with('-')
        || hostname.ends_with('.')
        || hostname.starts_with('.')
        || hostname.is_empty())
}

fn get_ip_addresses(host: &str) -> Vec<IpAddr> {
    let mut ips: Vec<IpAddr> = vec![];
    if is_valid_hostname(&host) {
        log::debug!("Found hostname: {}", host);
        ips = lookup_host(&host).unwrap();
    } else {
        log::debug!("Found IP address: {}", host);
        if let Ok(ip) = host.parse() {
            ips.push(ip);
        }
    }
    ips
}

pub fn is_host_in_network(host: &str, networks: &String) -> bool {
    let mut valid = false;
    let ips: Vec<std::net::IpAddr> = get_ip_addresses(host);
    log::debug!("Check if {:?} in networks {}", ips, networks);
    'outer: for network in networks.split(",") {
        if let Ok(nw) = IPAddress::parse(network) {
            for ip in &ips {
                if let Ok(ip) = IPAddress::parse(ip.to_string()) {
                    if nw.includes(&ip) {
                        valid = true;
                        log::info!("{} included in {}", host, nw.to_string());
                        break 'outer;
                    } else {
                        log::debug!("{} not included in {}", host, nw.to_string());
                    }
                }
            }
        } else {
            log::warn!("Invalid network - check '-n' parameter: {}", network);
        }
    }
    valid
}
