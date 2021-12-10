use crate::exclude::{is_connection_excluded, is_device_excluded, is_socket_excluded};
use crate::model::{Connection, Interface, Process};
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;

pub fn get_hostname(host: &String) -> String {
    let mut cmd = Command::new("ssh");
    cmd.arg(&host).arg("hostname");
    log::debug!("Cmd: {:?}", cmd);
    let output = cmd.output().expect("cannot call 'hostname' command");
    String::from_utf8(output.stdout)
        .expect("cannot convert cmd output to string")
        .trim()
        .to_string()
}

pub fn get_interfaces(host: &String, excludes: &Option<String>) -> Vec<Interface> {
    let mut interfaces = vec![];
    let mut cmd = Command::new("ssh");
    cmd.arg(&host)
        .arg("ip")
        .arg("--brief")
        .arg("address")
        .arg("show");
    log::debug!("Cmd: {:?}", cmd);
    let output = cmd.output().expect("cannot call 'ip a' command");
    for line in String::from_utf8(output.stdout)
        .expect("cannot convert cmd output to string")
        .lines()
    {
        let mut interface = Interface {
            name: "".to_string(),
            addresses: vec![],
        };
        for (index, field) in line.split_whitespace().enumerate() {
            match index {
                0 => interface.name = field.to_string(),
                1 => {}
                _ => interface.addresses.push(field.to_string()),
            }
        }
        if !is_device_excluded(&host, &interface.name, &excludes) {
            interfaces.push(interface);
        }
    }
    interfaces
}

fn extract_proc_name(input: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("\"(?P<proc>[[:word:]]+)\"").unwrap();
    }
    match RE
        .captures(input)
        .and_then(|cap| cap.name("proc").map(|proc| proc.as_str()))
    {
        None => String::from(""),
        Some(x) => x.to_string(),
    }
}

fn extract_addr_and_port(endpoint: &String, bindaddr: &mut String, port: &mut String) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<bindaddr>.+):(?P<port>\d{1,5})").unwrap();
    }
    if let Some(cap) = RE.captures(endpoint) {
        if let Some(addcapture) = cap.name("bindaddr") {
            *bindaddr = addcapture.as_str().to_string();
        }
        if let Some(portcapture) = cap.name("port") {
            *port = portcapture.as_str().to_string();
        }
    }
}

pub fn get_processes(host: &String, excludes: &Option<String>) -> Vec<Process> {
    let mut processes = vec![];
    let mut cmd = Command::new("ssh");
    cmd.arg(&host).arg("ss").arg("-tulpn");
    log::debug!("Cmd: {:?}", cmd);
    let output = cmd.output().expect("cannot call 'ss -tulpn' command");
    for line in String::from_utf8(output.stdout)
        .expect("cannot convert cmd output to string")
        .lines()
    {
        if line.contains(" LISTEN ") || line.contains(" UNCONN ") {
            let mut bindaddr = String::from("");
            let mut port = String::from("");
            let mut protocol = String::from("");
            let mut procname = String::from("");
            for (index, field) in line.split_whitespace().enumerate() {
                match index {
                    0 => protocol = field.to_string(),
                    4 => extract_addr_and_port(&field.to_string(), &mut bindaddr, &mut port),
                    6 => procname = extract_proc_name(field),
                    _ => {}
                }
            }
            let mut process = Process {
                name: procname,
                addresses: vec![],
            };
            if !is_socket_excluded(&host, &bindaddr, &port, &protocol, &excludes) {
                let address = String::from("") + &bindaddr + ":" + &port + "/" + &protocol;
                process.addresses.push(address);
            }
            // TODO: extra parameter to exclude specific processes
            if process.addresses.len() > 0 {
                processes.push(process);
            }
        }
    }
    processes
}

pub fn get_connections(host: &String, excludes: &Option<String>) -> Vec<Connection> {
    let mut connections = vec![];
    let mut cmd = Command::new("ssh");
    cmd.arg(&host).arg("ss").arg("-tuapn");
    log::debug!("Cmd: {:?}", cmd);
    let output = cmd.output().expect("cannot call 'ss -tuapn' command");
    for line in String::from_utf8(output.stdout)
        .expect("cannot convert cmd output to string")
        .lines()
    {
        if line.contains(" ESTAB ") {
            let mut localaddr = String::from("");
            let mut localport = String::from("");
            let mut remoteaddr = String::from("");
            let mut remoteport = String::from("");
            let mut procname = String::from("");
            let mut protocol = String::from("");
            for (index, field) in line.split_whitespace().enumerate() {
                match index {
                    0 => protocol = field.to_string(),
                    4 => extract_addr_and_port(&field.to_string(), &mut localaddr, &mut localport),
                    5 => {
                        extract_addr_and_port(&field.to_string(), &mut remoteaddr, &mut remoteport)
                    }
                    6 => procname = extract_proc_name(field),
                    _ => {}
                }
            }
            let connection = Connection {
                host: host.to_string(),
                process: procname.to_string(),
                local_addr: localaddr.to_string(),
                local_port: localport.to_string(),
                remote_addr: remoteaddr.to_string(),
                remote_port: remoteport.to_string(),
            };
            if !is_connection_excluded(
                &host,
                &remoteaddr,
                &remoteport,
                &localaddr,
                &localport,
                &protocol,
                &excludes,
            ) {
                connections.push(connection);
            }
        }
    }
    connections
}
