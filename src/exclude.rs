use lazy_static::lazy_static;
use regex::Regex;

fn get_exclusion_capture(exclude: &str) -> Option<regex::Captures<'_>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
(?P<host>[a-zA-Z0-9][a-zA-Z0-9.-]*[a-zA-Z0-9]){0,1}  # the hostname
(%(?P<device>[a-z0-9]+)){0,1} # the device
(:(?P<port>\d{1,5})){0,1}   # the port
(/(?P<protocol>[a-z0-9]+)){0,1}   # the protocol
"
        )
        .unwrap();
    }
    RE.captures(exclude)
}

pub fn is_host_excluded(host: &str, excludes: &Option<String>) -> bool {
    let mut excluded = false;
    if let Some(excludes) = excludes {
        for exclude in excludes.split(',') {
            if let Some(cap) = get_exclusion_capture(&exclude) {
                if let Some(name) = cap.name("host") {
                    if cap.name("device") == None
                        && cap.name("port") == None
                        && cap.name("protocol") == None
                    {
                        if name.as_str().eq(host) {
                            log::info!("Excluding host {} - rule: {}", host, exclude);
                            excluded = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    excluded
}

pub fn is_device_excluded(host: &str, device: &str, excludes: &Option<String>) -> bool {
    let mut excluded = false;
    if let Some(excludes) = excludes {
        for exclude in excludes.split(',') {
            if let Some(cap) = get_exclusion_capture(&exclude) {
                let hostname = match cap.name("host") {
                    None => "",
                    Some(x) => x.as_str(),
                };
                if let Some(devicename) = cap.name("device") {
                    if cap.name("port") == None && cap.name("protocol") == None {
                        if (hostname.eq(host) || hostname.eq("")) && devicename.as_str().eq(device)
                        {
                            log::info!("Excluding device {}%{} - rule: {}", host, device, exclude);
                            excluded = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    excluded
}

pub fn is_socket_excluded(
    host: &str,
    bindaddr: &str,
    port: &str,
    protocol: &str,
    excludes: &Option<String>,
) -> bool {
    let mut excluded = false;
    if let Some(excludes) = excludes {
        for exclude in excludes.split(',') {
            if let Some(cap) = get_exclusion_capture(&exclude) {
                let hostname = match cap.name("host") {
                    None => "",
                    Some(x) => x.as_str(),
                };
                let devicename = match cap.name("device") {
                    None => "",
                    Some(x) => x.as_str(),
                };
                let portname = match cap.name("port") {
                    None => "",
                    Some(x) => x.as_str(),
                };
                let protocolname = match cap.name("protocol") {
                    None => "",
                    Some(x) => x.as_str(),
                };
                if ((hostname.eq(host) || hostname.eq(bindaddr))
                    && portname.eq("")
                    && devicename.eq("")
                    && protocolname.eq(""))
                    || ((hostname.eq(host) || hostname.eq(bindaddr) || hostname.eq(""))
                        && portname.eq(port)
                        && devicename.eq("")
                        && (protocolname.eq(protocol) || protocolname.eq("")))
                    || ((hostname.eq(host) || hostname.eq(bindaddr) || hostname.eq(""))
                        && (portname.eq(port) || protocolname.eq(""))
                        && devicename.eq("")
                        && protocolname.eq(protocol))
                {
                    log::debug!(
                        "Excluding device [{}|{}]:{}/{} - rule: {}",
                        host,
                        bindaddr,
                        port,
                        protocol,
                        exclude
                    );
                    excluded = true;
                    break;
                }
            }
        }
    }
    excluded
}

pub fn is_connection_excluded(
    host: &str,
    localaddr: &str,
    localport: &str,
    remoteaddr: &str,
    remoteport: &str,
    protocol: &str,
    excludes: &Option<String>,
) -> bool {
    // TODO: get host/ip of remote and pass it - host should be tuple of (host, IP)
    let excluded = is_socket_excluded(host, localaddr, localport, protocol, excludes)
        && is_socket_excluded(host, remoteaddr, remoteport, protocol, excludes);
    if excluded {
        log::debug!(
            "Excluding connection [{}|{}]:{}/{} <-> {}:{}/{}",
            host,
            localaddr,
            localport,
            protocol,
            remoteaddr,
            remoteport,
            protocol,
        );
    }
    excluded
}
