use chorus_lib::config::Config;
use chorus_lib::ip::{Ban, IpData};
use chorus_lib::store::Store;
use dashmap::DashMap;
use hyper::server::conn::Http;
use lazy_static::lazy_static;
use std::net::IpAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::OnceLock;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::watch::Sender as WatchSender;

pub struct Globals {
    pub config: OnceLock<Config>,
    pub store: OnceLock<Store>,
    pub http_server: Http,
    pub rid: OnceLock<String>,

    /// This is a broadcast channel where new incoming events are advertised by their offset.
    /// Every handler needs to listen to it and check if the incoming event matches any
    /// subscribed fitlers for their client, and if so, send the event to their client under
    /// that subscription.
    pub new_events: BroadcastSender<usize>,

    pub num_clients: AtomicUsize,
    pub shutting_down: WatchSender<bool>,

    pub ip_data: DashMap<IpAddr, IpData>,
}

lazy_static! {
    pub static ref GLOBALS: Globals = {
        let mut http_server = hyper::server::conn::Http::new();
        http_server.http1_only(true);
        http_server.http1_keep_alive(true);

        let (new_events, _) = tokio::sync::broadcast::channel(512);
        let (shutting_down, _) = tokio::sync::watch::channel(false);

        Globals {
            config: OnceLock::new(),
            store: OnceLock::new(),
            http_server,
            rid: OnceLock::new(),
            new_events,
            num_clients: AtomicUsize::new(0),
            shutting_down,
            ip_data: DashMap::new(),
        }
    };
}

impl Globals {
    pub fn ban(ipaddr: std::net::IpAddr, bankind: Ban) -> u64 {
        if let Some(mut ipdata) = GLOBALS.ip_data.get_mut(&ipaddr) {
            ipdata.ban(bankind)
        } else {
            let mut ipdata = IpData::new();
            let seconds = ipdata.ban(bankind);
            GLOBALS.ip_data.insert(ipaddr, ipdata);
            seconds
        }
    }
}