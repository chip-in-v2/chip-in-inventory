/// Data access layer for etcd storage.
///
/// Implements the EtcdRepository to handle CRUD operations and
/// resource mapping for the inventory system.

use crate::ApiError;
use crate::models::{Hub, Realm, RoutingChain, Service, Subdomain, VirtualHost, Zone};
use etcd_client::{Client, GetOptions, SortOrder, SortTarget};
use serde::{de::DeserializeOwned, Serialize};

const REALM_KEY_PREFIX: &str = "realms/";

#[derive(Clone)]
pub struct EtcdRepository {
    client: Client,
}

impl EtcdRepository {
    pub async fn new(endpoints: &[&str]) -> Result<Self, etcd_client::Error> {
        let client = Client::connect(endpoints, None).await?;
        Ok(Self { client })
    }

    // --- Generic Helpers ---

    async fn save_resource<T: Serialize>(&self, key: &str, resource: &T) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let value = serde_json::to_string(resource)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    async fn get_resource<T: DeserializeOwned>(&self, key: &str) -> Result<T, ApiError> {
        let mut client = self.client.clone();
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let resource = serde_json::from_slice(kv.value())?;
            Ok(resource)
        } else {
            Err(ApiError::NotFound)
        }
    }

    async fn list_resources<T: DeserializeOwned>(&self, prefix: &str) -> Result<Vec<T>, ApiError> {
        let mut client = self.client.clone();
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix, Some(options)).await?;
        let resources = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                // Ensure that the remaining key part after removing prefix does not contain '/'.
                // This targets only direct children.
                if !key_str[prefix.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(resources)
    }

    async fn delete_resource(&self, key: &str) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    // --- Realm Methods ---

    fn realm_key(name: &str) -> String {
        format!("{}{}", REALM_KEY_PREFIX, name)
    }

    pub async fn save_realm(&self, realm: &Realm) -> Result<(), ApiError> {
        self.save_resource(&Self::realm_key(&realm.name), realm).await
    }

    pub async fn get_realm(&self, id: &str) -> Result<Realm, ApiError> {
        self.get_resource(&Self::realm_key(id)).await
    }

    pub async fn list_realms(&self) -> Result<Vec<Realm>, ApiError> {
        self.list_resources(REALM_KEY_PREFIX).await
    }

    pub async fn delete_realm(&self, id: &str) -> Result<bool, ApiError> {
        self.delete_resource(&Self::realm_key(id)).await
    }

    // --- Zone Methods ---

    fn zone_key(realm_id: &str, zone_id: &str) -> String {
        format!("{}{}/zones/{}", REALM_KEY_PREFIX, realm_id, zone_id)
    }

    fn zones_prefix(realm_id: &str) -> String {
        format!("{}{}/zones/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_zone(&self, realm_id: &str, zone: &Zone) -> Result<(), ApiError> {
        self.save_resource(&Self::zone_key(realm_id, &zone.name), zone).await
    }

    pub async fn get_zone(&self, realm_id: &str, zone_id: &str) -> Result<Zone, ApiError> {
        self.get_resource(&Self::zone_key(realm_id, zone_id)).await
    }

    pub async fn list_zones(&self, realm_id: &str) -> Result<Vec<Zone>, ApiError> {
        self.list_resources(&Self::zones_prefix(realm_id)).await
    }

    pub async fn delete_zone(&self, realm_id: &str, zone_id: &str) -> Result<bool, ApiError> {
        self.delete_resource(&Self::zone_key(realm_id, zone_id)).await
    }

    // --- Subdomain Methods ---

    fn subdomain_key(realm_id: &str, zone_id: &str, subdomain_id: &str) -> String {
        format!(
            "{}{}/zones/{}/subdomains/{}",
            REALM_KEY_PREFIX, realm_id, zone_id, subdomain_id
        )
    }

    fn subdomains_prefix(realm_id: &str, zone_id: &str) -> String {
        format!(
            "{}{}/zones/{}/subdomains/",
            REALM_KEY_PREFIX, realm_id, zone_id
        )
    }

    pub async fn save_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain: &Subdomain,
    ) -> Result<(), ApiError> {
        self.save_resource(&Self::subdomain_key(realm_id, zone_id, &subdomain.name), subdomain).await
    }

    pub async fn get_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain_id: &str,
    ) -> Result<Subdomain, ApiError> {
        self.get_resource(&Self::subdomain_key(realm_id, zone_id, subdomain_id)).await
    }

    pub async fn list_subdomains(
        &self,
        realm_id: &str,
        zone_id: &str,
    ) -> Result<Vec<Subdomain>, ApiError> {
        self.list_resources(&Self::subdomains_prefix(realm_id, zone_id)).await
    }

    pub async fn delete_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::subdomain_key(realm_id, zone_id, subdomain_id)).await
    }

    // --- VirtualHost Methods ---

    fn virtual_host_key(realm_id: &str, virtual_host_id: &str) -> String {
        format!(
            "{}{}/virtual-hosts/{}",
            REALM_KEY_PREFIX, realm_id, virtual_host_id
        )
    }

    fn virtual_hosts_prefix(realm_id: &str) -> String {
        format!("{}{}/virtual-hosts/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_virtual_host(
        &self,
        realm_id: &str,
        virtual_host: &VirtualHost,
    ) -> Result<(), ApiError> {
        self.save_resource(&Self::virtual_host_key(realm_id, &virtual_host.name), virtual_host).await
    }

    pub async fn get_virtual_host(
        &self,
        realm_id: &str,
        virtual_host_id: &str,
    ) -> Result<VirtualHost, ApiError> {
        self.get_resource(&Self::virtual_host_key(realm_id, virtual_host_id)).await
    }

    pub async fn list_virtual_hosts(&self, realm_id: &str) -> Result<Vec<VirtualHost>, ApiError> {
        self.list_resources(&Self::virtual_hosts_prefix(realm_id)).await
    }

    pub async fn delete_virtual_host(
        &self,
        realm_id: &str,
        virtual_host_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::virtual_host_key(realm_id, virtual_host_id)).await
    }

    // --- RoutingChain Methods ---

    fn routing_chain_key(realm_id: &str) -> String {
        format!(
            "{}{}/routing-chain",
            REALM_KEY_PREFIX, realm_id
        )
    }

    pub async fn save_routing_chain(
        &self,
        realm_id: &str,
        routing_chain: &RoutingChain,
    ) -> Result<(), ApiError> {
        self.save_resource(&Self::routing_chain_key(realm_id), routing_chain).await
    }

    pub async fn get_routing_chain(
        &self,
        realm_id: &str,
    ) -> Result<RoutingChain, ApiError> {
        self.get_resource(&Self::routing_chain_key(realm_id)).await
    }

    pub async fn list_routing_chains(&self, realm_id: &str) -> Result<Vec<RoutingChain>, ApiError> {
        match self.get_routing_chain(realm_id).await {
            Ok(rc) => Ok(vec![rc]),
            Err(ApiError::NotFound) => Ok(vec![]),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_routing_chain(
        &self,
        realm_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::routing_chain_key(realm_id)).await
    }

    // --- Hub Methods ---

    fn hub_key(realm_id: &str, hub_id: &str) -> String {
        format!("{}{}/hubs/{}", REALM_KEY_PREFIX, realm_id, hub_id)
    }

    fn hubs_prefix(realm_id: &str) -> String {
        format!("{}{}/hubs/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_hub(&self, realm_id: &str, hub: &Hub) -> Result<(), ApiError> {
        self.save_resource(&Self::hub_key(realm_id, &hub.name), hub).await
    }

    pub async fn get_hub(&self, realm_id: &str, hub_id: &str) -> Result<Hub, ApiError> {
        self.get_resource(&Self::hub_key(realm_id, hub_id)).await
    }

    pub async fn list_hubs(&self, realm_id: &str) -> Result<Vec<Hub>, ApiError> {
        self.list_resources(&Self::hubs_prefix(realm_id)).await
    }

    pub async fn delete_hub(&self, realm_id: &str, hub_id: &str) -> Result<bool, ApiError> {
        self.delete_resource(&Self::hub_key(realm_id, hub_id)).await
    }

    // --- Service Methods ---

    fn service_key(realm_id: &str, hub_id: &str, service_id: &str) -> String {
        format!(
            "{}{}/hubs/{}/services/{}",
            REALM_KEY_PREFIX, realm_id, hub_id, service_id
        )
    }

    fn services_prefix(realm_id: &str, hub_id: &str) -> String {
        format!("{}{}/hubs/{}/services/", REALM_KEY_PREFIX, realm_id, hub_id)
    }

    pub async fn save_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service: &Service,
    ) -> Result<(), ApiError> {
        self.save_resource(&Self::service_key(realm_id, hub_id, &service.name), service).await
    }

    pub async fn get_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service_id: &str,
    ) -> Result<Service, ApiError> {
        self.get_resource(&Self::service_key(realm_id, hub_id, service_id)).await
    }

    pub async fn list_services(&self, realm_id: &str, hub_id: &str) -> Result<Vec<Service>, ApiError> {
        self.list_resources(&Self::services_prefix(realm_id, hub_id)).await
    }

    pub async fn delete_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::service_key(realm_id, hub_id, service_id)).await
    }
}
