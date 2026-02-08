use crate::api_client::{ApiClient};
use netlink_packet_route::{
    address::AddressAttribute,
    address::AddressMessage,
    RouteNetlinkMessage,
};
use netlink_packet_core::{NetlinkPayload,NetlinkMessage};
use rtnetlink::new_connection;
use futures::{StreamExt, TryStreamExt};
use std::{net::{IpAddr, Ipv4Addr}};
use anyhow::{Result};
use futures_channel::mpsc::UnboundedReceiver;
use netlink_sys::SocketAddr;

pub(crate) struct IpChangeListener {
    last_ip: Option<String>,
    api: ApiClient,
    link_index: u32,
    messages: UnboundedReceiver<(NetlinkMessage<RouteNetlinkMessage>, SocketAddr)>,
    handle: rtnetlink::Handle
}

impl IpChangeListener {
    pub(crate) async fn init(api: ApiClient, interface: &str) -> Result<Self> {
        let (connection, handle, messages) = new_connection()?;
        tokio::spawn(connection);

        // resolve interface
        let mut links = handle
            .link()
            .get()
            .match_name(interface.to_string())
            .execute();

        let link = links
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("interface not found"))??;

        let link_index = link.header.index;

        Ok(Self {
            last_ip: None,
            api,
            link_index,
            messages,
            handle,
        })
    }

    pub(crate) async fn start(self) -> Result<tokio::task::JoinHandle<Result<(), anyhow::Error>>> {
        let handle = tokio::spawn(async move {
            match self.run().await {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("IP change listener stopped: {e}");
                    Err(e)
                }
            }
        });
        
        Ok(handle)
    }

    pub(crate) async fn get_initial_ipv4(&self) -> Option<Ipv4Addr> {
        let mut addrs = self.handle
            .address()
            .get()
            .set_link_index_filter(self.link_index)
            .execute();

        while let Ok(Some(addr)) = addrs.try_next().await {
            if let Some(ip) = extract_ipv4(&addr) {
                return Some(ip);
            }
        }

        None
    }

    async fn run(mut self) -> Result<()> {
        while let Some((msg, _)) = self.messages.next().await {
            let NetlinkPayload::InnerMessage(inner) = msg.payload else {
                continue;
            };

            let (addr, event) = match inner {
                RouteNetlinkMessage::NewAddress(a) => (a, "add"),
                RouteNetlinkMessage::DelAddress(a) => (a, "del"),
                _ => continue,
            };

            if addr.header.index != self.link_index {
                continue;
            }

            let Some(ip) = extract_ipv4(&addr).map(|ip| ip.to_string()) else {
                continue;
            };

            if self.last_ip.as_deref() == Some(&ip) {
                continue;
            }

            self.last_ip = Some(ip.clone());

            // TODO: specific endpoint for IP update/deletion
            if let Err(e) = self.api.update_ip(
                self.last_ip.clone(),
                event.to_string(),
            ).await {
                eprintln!("Failed to report IP change: {e}");
            }
        }

        Err(anyhow::anyhow!("IP changes subscription ended"))
    }
}

fn extract_ipv4(msg: &AddressMessage) -> Option<Ipv4Addr> {
    for attr in &msg.attributes {
        if let AddressAttribute::Address(ip) = attr {
            if let IpAddr::V4(v4) = ip {
                return Some(*v4);
            }
        }
    }
    None
}