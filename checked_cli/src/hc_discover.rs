use holochain_client::AdminWebsocket;
use proc_ctl::{PortQuery, ProcQuery, ProtocolPort};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

/// Search for a Holochain process interactively. For each process that is found check what ports it
/// is listening on. Will attempt to return when there is only one option to select and will prompt
/// the user to select a process and port when there are multiple options.
pub(crate) async fn interactive_discover_holochain() -> anyhow::Result<u16> {
    let query = ProcQuery::new().process_name("holochain");

    let processes = query.list_processes()?;

    let possible_processes_with_ports = processes
        .into_iter()
        .filter_map(|p| {
            let port_query = PortQuery::new().ip_v4_only().tcp_only().process_id(p.pid);

            match port_query.execute() {
                Ok(ports) => {
                    let tcp_ports = ports
                        .into_iter()
                        .filter_map(|p| match p {
                            ProtocolPort::Tcp(p) => Some(p),
                            _ => None,
                        })
                        .collect::<Vec<_>>();

                    if tcp_ports.is_empty() {
                        None
                    } else {
                        Some((p, tcp_ports))
                    }
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if possible_processes_with_ports.is_empty() {
        // Handle the case that no Holochain processes were found
        anyhow::bail!("No Holochain processes found.");
    } else if possible_processes_with_ports
        .iter()
        .all(|(_, ports)| ports.is_empty())
    {
        // Handle the case that Holochain processes were found but all have no ports open
        anyhow::bail!(
            "Found {} Holochain processes found but none have any ports open.",
            possible_processes_with_ports.len()
        );
    } else if possible_processes_with_ports.len() == 1
        && possible_processes_with_ports[0].1.len() == 1
    {
        // Perfect match case with one instance of Holochain and one port open
        return Ok(possible_processes_with_ports[0].1[0]);
    }

    println!("{:?}", possible_processes_with_ports);

    let selected = dialoguer::Select::new()
        .with_prompt("Pick a Holochain process")
        .items(
            &possible_processes_with_ports
                .iter()
                .map(|(p, ports)| {
                    format!(
                        "Process ID: {}, launched with arguments: {:?}, has {} ports open",
                        p.pid,
                        p.cmd,
                        ports.len()
                    )
                })
                .collect::<Vec<_>>(),
        )
        .interact()?;

    let (_, ports) = &possible_processes_with_ports[selected];

    // Filter down to the ports that respond to admin requests and ignore anything that is likely
    // to be an app port.
    let mut admin_ports = vec![];
    for port in ports {
        if is_admin_port(*port).await {
            println!("Port {} is an admin port", port);
            admin_ports.push(*port);
        }
    }

    if admin_ports.is_empty() {
        anyhow::bail!("No admin ports open for selected Holochain process");
    } else if admin_ports.len() == 1 {
        return Ok(admin_ports[0]);
    }

    let port_index = dialoguer::Select::new()
        .with_prompt("Choose a port")
        .items(
            &admin_ports
                .iter()
                .map(|p| format!("Port: {}", p))
                .collect::<Vec<_>>(),
        )
        .interact()?;

    Ok(admin_ports[port_index])
}

async fn is_admin_port(port: u16) -> bool {
    let ipv4_addr: SocketAddr = (Ipv4Addr::LOCALHOST, port).into();
    let ipv6_addr: SocketAddr = (Ipv6Addr::LOCALHOST, port).into();

    let mut client = match AdminWebsocket::connect(vec![ipv4_addr, ipv6_addr].as_slice()).await {
        Ok(client) => client,
        Err(_) => return false,
    };

    client.list_apps(None).await.is_ok()
}
