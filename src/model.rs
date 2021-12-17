use crate::cli::{get_connections, get_hostname, get_interfaces, get_processes};
use crate::exclude::is_host_excluded;
use crate::graph::generate_graph;
use crate::network::is_host_in_network;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub machines: Vec<Machine>,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    pub hostname: String,
    pub interfaces: Vec<Interface>,
    pub processes: Vec<Process>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub host: String,
    pub process: String,
    pub local_addr: String,
    pub local_port: String,
    pub remote_addr: String,
    pub remote_port: String,
}

impl Model {
    pub fn new() -> Model {
        Model {
            machines: vec![],
            connections: vec![],
        }
    }
    pub fn add_machine(&mut self, host: &String, excludes: &Option<String>, networks: &String) {
        if !is_host_in_network(&host, &networks) || is_host_excluded(&host, &excludes) {
            return;
        }
        let hostname = get_hostname(&host);
        if !is_host_in_network(&hostname, &networks) || is_host_excluded(&hostname, &excludes) {
            return;
        }
        for item in &self.machines {
            if item.hostname == hostname {
                return;
            }
        }
        let interfaces = get_interfaces(&host, &excludes);
        let processes = get_processes(&host, &excludes);
        self.machines.push(Machine {
            hostname,
            interfaces,
            processes,
        });
        let mut connections = get_connections(&host, &excludes);
        // TODO: add step to move connection addresses to existing interfaces
        self.connections.append(&mut connections);
        // TODO: call self.add_machine() with all remote hosts
        // using connection src/dest
        for remote_host in &connections {
            self.add_machine(&remote_host.remote_addr, &excludes, &networks);
        }
    }
    pub fn generate(&self, filename: &Option<String>) {
        let output = generate_graph(&self);
        if let Some(filename) = filename {
            let mut file = File::create(filename).unwrap();
            file.write_all(output.as_bytes()).unwrap();
        } else {
            println!("{}", output);
        }
    }
}
