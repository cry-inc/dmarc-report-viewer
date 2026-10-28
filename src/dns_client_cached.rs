use crate::cache_map::CacheMap;
use crate::dns_client::DnsClient;
use anyhow::Result;
use std::{net::IpAddr, sync::Arc};
use tokio::sync::RwLock;

pub struct DnsClientCached {
    dns_client: DnsClient,
    cache: Arc<RwLock<CacheMap<IpAddr, Option<String>>>>,
}

impl DnsClientCached {
    pub fn new(dns_client: DnsClient, max_cache_size: usize) -> Self {
        Self {
            dns_client,
            cache: Arc::new(RwLock::new(
                CacheMap::new(max_cache_size).expect("Failed to create cache"),
            )),
        }
    }

    pub async fn host_from_ip(&self, ip: IpAddr) -> Result<Option<String>> {
        // First check cache
        {
            let locked = self.cache.read().await;
            if let Some(cached) = locked.get(&ip) {
                return Ok(cached.clone());
            }
        }

        // Otherwise send real query over network
        let result = self.dns_client.host_from_ip(ip).await;

        // Cache any result that is not an error
        if let Ok(response) = &result {
            let mut locked = self.cache.write().await;
            locked.insert(ip, response.clone());
        }

        result
    }
}
