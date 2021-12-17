use crate::model::{Connection, Interface, Machine, Model, Process};
use dot_writer::{Attributes, Color, DotWriter, Scope, Shape, Style};
use lazy_static::lazy_static;
use regex::Regex;

fn ip_only(input: &String) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"/\w+$").unwrap();
        static ref RE2: Regex = Regex::new(r":\d{1,5}$").unwrap();
        static ref RE3: Regex = Regex::new(r"/\d+").unwrap();
        static ref RE4: Regex = Regex::new(r"%[0-9a-z]+").unwrap();
    }
    let mut result = RE1.replace(input, "").to_string();
    result = RE2.replace(&result, "").to_string();
    log::debug!("{} - {}", result, RE3.is_match(&result));
    result = RE3.replace(&result, "").to_string();
    log::debug!("Sanitized IP: {} --> {}", input, result);
    result = RE4.replace(&result, "").to_string();
    result
}
fn sanitiza_label(input: &String) -> String {
    let output = input.replace("-", "").replace("@", "").replace(":", "_");
    output
}

fn generate_machine_node(digraph: &mut Scope, machine: &Machine) {
    {
        let mut cluster = digraph.cluster();
        cluster.set_style(Style::Filled);
        cluster.set_color(Color::LightGrey);
        cluster.set_rank_direction(dot_writer::RankDirection::BottomTop);
        cluster
            .node_attributes()
            .set_style(Style::Filled)
            .set_color(Color::White);
        cluster.set_label(&sanitiza_label(&machine.hostname));
        for interface in &machine.interfaces {
            let mut device = String::from(&sanitiza_label(&machine.hostname));
            device.push_str(&sanitiza_label(&interface.name));
            let mut label = String::from(format!("<{}> {}", ip_only(&device), interface.name));
            for addr in &interface.addresses {
                label.push_str(" | ");
                label.push_str(&format!("<{}> {}", sanitiza_label(&ip_only(&addr)), &addr));
            }
            cluster
                .node_named(device)
                .set_label(&label)
                .set_shape(Shape::Record);
        }
        for process in &machine.processes {
            let mut label = String::from(&process.name);
            let mut name = String::from(&machine.hostname);
            if label.eq("") {
                label = String::from("_unknown_");
            }
            name.push_str(&label);
            cluster
                .node_named(&name)
                .set_label(&label)
                .set_shape(Shape::Circle);
            for bind in &process.addresses {
                let bindport = ip_only(&bind);
                if bindport.eq("127.0.0.1") || bindport.eq("[::1]") {
                    // TODO: treat local binds
                    continue;
                }
                if bindport.eq("*") || bindport.eq("0.0.0.0")  || bindport.eq("[::1]"){
                    for interface in &machine.interfaces {
                        // TODO: check IPv4/6 and add clap options
                        for addr in &interface.addresses {
                            cluster.edge(
                                &name,
                                ip_only(&format!(
                                    "{}{}:\"{}\"",
                                    machine.hostname,
                                    sanitiza_label(&ip_only(&interface.name)),
                                    sanitiza_label(&ip_only(&addr))
                                )),
                            );
                        }
                    }
                } else {
                    cluster.edge(
                        &name,
                        ip_only(&format!(
                            "{}{}:\"{}\"",
                            machine.hostname,
                            "TODO",
                            sanitiza_label(&ip_only(&bindport))
                        )),
                    );
                }
            }
        }
    }
}

pub fn generate_graph(model: &Model) -> String {
    let mut output_bytes = Vec::new();
    {
        let mut writer = DotWriter::from(&mut output_bytes);
        writer.set_pretty_print(true);
        let mut digraph = writer.digraph();
        digraph.set_rank_direction(dot_writer::RankDirection::LeftRight);
        for machine in &model.machines {
            generate_machine_node(&mut digraph, &machine)
        }
    }
    String::from_utf8(output_bytes).unwrap()
}
