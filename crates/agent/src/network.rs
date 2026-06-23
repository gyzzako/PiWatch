use crate::api_client::{ApiClient};
use netlink_packet_route::{
    RouteNetlinkMessage, address::{AddressAttribute, AddressMessage},
};
use netlink_packet_core::{NetlinkPayload,NetlinkMessage};
use rtnetlink::{constants::RTMGRP_IPV4_IFADDR, new_connection};
use futures::{StreamExt, TryStreamExt};
use tokio::sync::Mutex;
use std::{net::{IpAddr, Ipv4Addr}, sync::Arc};
use anyhow::{Result};
use futures_channel::mpsc::UnboundedReceiver;
use netlink_sys::{AsyncSocket, SocketAddr};
use core_watch::logging::{debug, error, info};

pub(crate) struct IpChangeListener {
    api: ApiClient,
    link_index: u32,
    messages: Mutex<UnboundedReceiver<(NetlinkMessage<RouteNetlinkMessage>, SocketAddr)>>,
    curr_ip: Mutex<Option<Ipv4Addr>>,
}

impl IpChangeListener {
    pub(crate) async fn init(api: ApiClient, interface: &str) -> Result<Self> {
        let (mut connection, handle, messages) = new_connection()?;
        
        connection.socket_mut().socket_mut().bind(
            &SocketAddr::new(0, RTMGRP_IPV4_IFADDR as u32)
        )?;
        
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
            api,
            link_index,
            messages: Mutex::new(messages),
            curr_ip: Mutex::new(get_initial_ipv4(&handle, link_index).await),
        })
    }

    pub(crate) fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            if let Err(e) = self.run().await {
                error!("IP change listener stopped: {e}");
            }
        })
    }

    pub(crate) async fn get_current_ip(&self) -> Option<Ipv4Addr> {
        *self.curr_ip.lock().await
    }

    async fn run(self: Arc<Self>) -> Result<()> {
        loop {
            let msg_opt = {
                let mut rx = self.messages.lock().await;
                rx.next().await
            };

            let Some((msg, _)) = msg_opt else {
                break;
            };
            let NetlinkPayload::InnerMessage(inner) = msg.payload else {
                continue;
            };
            
            debug!("RAW MESSAGE: {:?}", inner);
            let addr = match inner {
                RouteNetlinkMessage::NewAddress(a) => a,
                _ => continue,
            };

            if addr.header.index != self.link_index {
                continue;
            }

            let Some(ip) = extract_ipv4(&addr).map(|ip| ip.to_string()) else {
                continue;
            };

            if self.curr_ip.lock().await.as_ref().map_or(false, |curr| curr.to_string() == ip) {
                continue;
            }

            info!("Detected IP change from {} to {}", self.curr_ip.lock().await.as_ref().map_or_else(|| "unknown".into(), |ip| ip.to_string()), ip);

            match self.api.reconcile_ip(Some(ip.clone())).await {
                Ok(_) => *self.curr_ip.lock().await = Some(ip.parse()?),
                Err(e) => error!("Failed to report IP change: {e}"),
                
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

async fn get_initial_ipv4(handle: &rtnetlink::Handle, link_index: u32) -> Option<Ipv4Addr> {
    let mut addrs = handle
        .address()
        .get()
        .set_link_index_filter(link_index)
        .execute();

    while let Ok(Some(addr)) = addrs.try_next().await {
        if let Some(ip) = extract_ipv4(&addr) {
            return Some(ip);
        }
    }
    
    None
}