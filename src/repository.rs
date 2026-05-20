//! Data access layer for etcd storage.
//!
//! Implements the EtcdRepository to handle CRUD operations and
//! resource mapping for the inventory system.
use crate::ApiError;
use crate::models::{Hub, Realm, RoutingChain, Service, Subdomain, VirtualHost, Zone};
use etcd_client::{
    Client, Compare, CompareOp, DeleteOptions, GetOptions, SortOrder, SortTarget, Txn, TxnOp,
    TxnOpResponse,
};
use serde::{Serialize, de::DeserializeOwned};

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

    async fn cascade_delete(&self, exact_key: &str, dir_prefix: &str) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let op_del_exact = TxnOp::delete(exact_key, None);
        let op_del_prefix = TxnOp::delete(dir_prefix, Some(DeleteOptions::new().with_prefix()));
        let txn = Txn::new().and_then(vec![op_del_exact, op_del_prefix]);
        let resp = client.txn(txn).await?;
        if let Some(TxnOpResponse::Delete(del_resp)) = resp.op_responses().first() {
            Ok(del_resp.deleted() > 0)
        } else {
            Ok(false)
        }
    }

    async fn delete_resource(&self, key: &str) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    async fn get_resource_with_revision<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<(T, i64), ApiError> {
        let mut client = self.client.clone();
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let resource = serde_json::from_slice(kv.value())?;
            Ok((resource, kv.mod_revision()))
        } else {
            Err(ApiError::NotFound)
        }
    }

    async fn save_resource_conditional<T: Serialize>(
        &self,
        key: &str,
        resource: &T,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let value = serde_json::to_string(resource)?;
        let compare = if expected_revision == 0 {
            Compare::version(key, CompareOp::Equal, 0)
        } else {
            Compare::mod_revision(key, CompareOp::Equal, expected_revision)
        };
        let txn = Txn::new()
            .when(vec![compare])
            .and_then(vec![TxnOp::put(key, value, None)]);
        let resp = client.txn(txn).await?;
        Ok(resp.succeeded())
    }

    // --- Realm Methods ---

    fn realm_key(name: &str) -> String {
        format!("{}{}", REALM_KEY_PREFIX, name)
    }

    fn realm_dir(name: &str) -> String {
        format!("{}{}/", REALM_KEY_PREFIX, name)
    }

    pub async fn save_realm(&self, realm: &Realm) -> Result<(), ApiError> {
        self.save_resource(&Self::realm_key(&realm.name), realm)
            .await
    }

    pub async fn save_realm_conditional(
        &self,
        realm: &Realm,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        self.save_resource_conditional(&Self::realm_key(&realm.name), realm, expected_revision)
            .await
    }

    pub async fn get_realm(&self, id: &str) -> Result<Realm, ApiError> {
        self.get_resource(&Self::realm_key(id)).await
    }

    pub async fn get_realm_with_revision(&self, id: &str) -> Result<(Realm, i64), ApiError> {
        self.get_resource_with_revision(&Self::realm_key(id)).await
    }

    pub async fn list_realms(&self) -> Result<Vec<Realm>, ApiError> {
        self.list_resources(REALM_KEY_PREFIX).await
    }

    pub async fn delete_realm(&self, id: &str) -> Result<bool, ApiError> {
        self.cascade_delete(&Self::realm_key(id), &Self::realm_dir(id))
            .await
    }

    // --- Zone Methods ---

    fn zone_key(realm_id: &str, zone_id: &str) -> String {
        format!("{}{}/zones/{}", REALM_KEY_PREFIX, realm_id, zone_id)
    }

    fn zone_dir(realm_id: &str, zone_id: &str) -> String {
        format!("{}{}/zones/{}/", REALM_KEY_PREFIX, realm_id, zone_id)
    }

    fn zones_prefix(realm_id: &str) -> String {
        format!("{}{}/zones/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_zone(&self, realm_id: &str, zone: &Zone) -> Result<(), ApiError> {
        self.save_resource(&Self::zone_key(realm_id, &zone.name), zone)
            .await
    }

    pub async fn save_zone_conditional(
        &self,
        realm_id: &str,
        zone: &Zone,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        self.save_resource_conditional(
            &Self::zone_key(realm_id, &zone.name),
            zone,
            expected_revision,
        )
        .await
    }

    pub async fn get_zone(&self, realm_id: &str, zone_id: &str) -> Result<Zone, ApiError> {
        self.get_resource(&Self::zone_key(realm_id, zone_id)).await
    }

    pub async fn get_zone_with_revision(
        &self,
        realm_id: &str,
        zone_id: &str,
    ) -> Result<(Zone, i64), ApiError> {
        self.get_resource_with_revision(&Self::zone_key(realm_id, zone_id))
            .await
    }

    pub async fn list_zones(&self, realm_id: &str) -> Result<Vec<Zone>, ApiError> {
        self.list_resources(&Self::zones_prefix(realm_id)).await
    }

    pub async fn delete_zone(&self, realm_id: &str, zone_id: &str) -> Result<bool, ApiError> {
        self.cascade_delete(
            &Self::zone_key(realm_id, zone_id),
            &Self::zone_dir(realm_id, zone_id),
        )
        .await
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
        self.save_resource(
            &Self::subdomain_key(realm_id, zone_id, &subdomain.name),
            subdomain,
        )
        .await
    }

    pub async fn save_subdomain_conditional(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain: &Subdomain,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        self.save_resource_conditional(
            &Self::subdomain_key(realm_id, zone_id, &subdomain.name),
            subdomain,
            expected_revision,
        )
        .await
    }

    pub async fn get_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain_id: &str,
    ) -> Result<Subdomain, ApiError> {
        self.get_resource(&Self::subdomain_key(realm_id, zone_id, subdomain_id))
            .await
    }

    pub async fn get_subdomain_with_revision(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain_id: &str,
    ) -> Result<(Subdomain, i64), ApiError> {
        self.get_resource_with_revision(&Self::subdomain_key(realm_id, zone_id, subdomain_id))
            .await
    }

    pub async fn list_subdomains(
        &self,
        realm_id: &str,
        zone_id: &str,
    ) -> Result<Vec<Subdomain>, ApiError> {
        self.list_resources(&Self::subdomains_prefix(realm_id, zone_id))
            .await
    }

    pub async fn list_all_subdomains_in_realm(
        &self,
        realm_id: &str,
    ) -> Result<Vec<Subdomain>, ApiError> {
        let mut client = self.client.clone();
        let prefix = format!("{}{}/zones/", REALM_KEY_PREFIX, realm_id);
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix, Some(options)).await?;
        let mut subdomains = Vec::new();
        for kv in resp.kvs() {
            if let Ok(key_str) = kv.key_str()
                && let Some(pos) = key_str.find("/subdomains/")
            {
                let sub_part = &key_str[pos + "/subdomains/".len()..];
                if !sub_part.contains('/')
                    && let Ok(mut subdomain) = serde_json::from_slice::<Subdomain>(kv.value())
                {
                    let parts: Vec<&str> = key_str.split('/').collect();
                    if parts.len() >= 6 {
                        let zone_id = parts[3];
                        subdomain.urn =
                            Some(Subdomain::generate_urn(realm_id, zone_id, &subdomain.name));
                        subdomain.realm = Some(Realm::generate_urn(realm_id));
                        subdomain.zone = Some(Zone::generate_urn(realm_id, zone_id));
                        subdomain.fqdn = Some(Subdomain::generate_fqdn(zone_id, &subdomain.name));
                    }
                    subdomains.push(subdomain);
                }
            }
        }
        Ok(subdomains)
    }

    pub async fn delete_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::subdomain_key(realm_id, zone_id, subdomain_id))
            .await
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
        self.save_resource(
            &Self::virtual_host_key(realm_id, &virtual_host.name),
            virtual_host,
        )
        .await
    }

    pub async fn save_virtual_host_conditional(
        &self,
        realm_id: &str,
        virtual_host: &VirtualHost,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        self.save_resource_conditional(
            &Self::virtual_host_key(realm_id, &virtual_host.name),
            virtual_host,
            expected_revision,
        )
        .await
    }

    pub async fn get_virtual_host(
        &self,
        realm_id: &str,
        virtual_host_id: &str,
    ) -> Result<VirtualHost, ApiError> {
        self.get_resource(&Self::virtual_host_key(realm_id, virtual_host_id))
            .await
    }

    pub async fn get_virtual_host_with_revision(
        &self,
        realm_id: &str,
        virtual_host_id: &str,
    ) -> Result<(VirtualHost, i64), ApiError> {
        self.get_resource_with_revision(&Self::virtual_host_key(realm_id, virtual_host_id))
            .await
    }

    pub async fn list_virtual_hosts(&self, realm_id: &str) -> Result<Vec<VirtualHost>, ApiError> {
        self.list_resources(&Self::virtual_hosts_prefix(realm_id))
            .await
    }

    pub async fn delete_virtual_host(
        &self,
        realm_id: &str,
        virtual_host_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::virtual_host_key(realm_id, virtual_host_id))
            .await
    }

    // --- RoutingChain Methods ---

    fn routing_chain_key(realm_id: &str, routing_chain_id: &str) -> String {
        format!(
            "{}{}/routing-chains/{}",
            REALM_KEY_PREFIX, realm_id, routing_chain_id
        )
    }

    fn routing_chains_prefix(realm_id: &str) -> String {
        format!("{}{}/routing-chains/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_routing_chain(
        &self,
        realm_id: &str,
        routing_chain: &RoutingChain,
    ) -> Result<(), ApiError> {
        self.save_resource(
            &Self::routing_chain_key(realm_id, &routing_chain.name),
            routing_chain,
        )
        .await
    }

    pub async fn save_routing_chain_conditional(
        &self,
        realm_id: &str,
        routing_chain: &RoutingChain,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        self.save_resource_conditional(
            &Self::routing_chain_key(realm_id, &routing_chain.name),
            routing_chain,
            expected_revision,
        )
        .await
    }

    pub async fn get_routing_chain(
        &self,
        realm_id: &str,
        routing_chain_id: &str,
    ) -> Result<RoutingChain, ApiError> {
        self.get_resource(&Self::routing_chain_key(realm_id, routing_chain_id))
            .await
    }

    pub async fn get_routing_chain_with_revision(
        &self,
        realm_id: &str,
        routing_chain_id: &str,
    ) -> Result<(RoutingChain, i64), ApiError> {
        self.get_resource_with_revision(&Self::routing_chain_key(realm_id, routing_chain_id))
            .await
    }

    pub async fn list_routing_chains(&self, realm_id: &str) -> Result<Vec<RoutingChain>, ApiError> {
        self.list_resources(&Self::routing_chains_prefix(realm_id))
            .await
    }

    pub async fn delete_routing_chain(
        &self,
        realm_id: &str,
        routing_chain_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::routing_chain_key(realm_id, routing_chain_id))
            .await
    }

    // --- Hub Methods ---

    fn hub_key(realm_id: &str, hub_id: &str) -> String {
        format!("{}{}/hubs/{}", REALM_KEY_PREFIX, realm_id, hub_id)
    }

    fn hub_dir(realm_id: &str, hub_id: &str) -> String {
        format!("{}{}/hubs/{}/", REALM_KEY_PREFIX, realm_id, hub_id)
    }

    fn hubs_prefix(realm_id: &str) -> String {
        format!("{}{}/hubs/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_hub(&self, realm_id: &str, hub: &Hub) -> Result<(), ApiError> {
        self.save_resource(&Self::hub_key(realm_id, &hub.name), hub)
            .await
    }

    pub async fn get_hub(&self, realm_id: &str, hub_id: &str) -> Result<Hub, ApiError> {
        self.get_resource(&Self::hub_key(realm_id, hub_id)).await
    }

    pub async fn save_hub_conditional(
        &self,
        realm_id: &str,
        hub: &Hub,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        self.save_resource_conditional(&Self::hub_key(realm_id, &hub.name), hub, expected_revision)
            .await
    }

    pub async fn get_hub_with_revision(
        &self,
        realm_id: &str,
        hub_id: &str,
    ) -> Result<(Hub, i64), ApiError> {
        self.get_resource_with_revision(&Self::hub_key(realm_id, hub_id))
            .await
    }

    pub async fn list_hubs(&self, realm_id: &str) -> Result<Vec<Hub>, ApiError> {
        self.list_resources(&Self::hubs_prefix(realm_id)).await
    }

    pub async fn delete_hub(&self, realm_id: &str, hub_id: &str) -> Result<bool, ApiError> {
        self.cascade_delete(
            &Self::hub_key(realm_id, hub_id),
            &Self::hub_dir(realm_id, hub_id),
        )
        .await
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
        self.save_resource(&Self::service_key(realm_id, hub_id, &service.name), service)
            .await
    }

    pub async fn save_service_conditional(
        &self,
        realm_id: &str,
        hub_id: &str,
        service: &Service,
        expected_revision: i64,
    ) -> Result<bool, ApiError> {
        self.save_resource_conditional(
            &Self::service_key(realm_id, hub_id, &service.name),
            service,
            expected_revision,
        )
        .await
    }

    pub async fn get_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service_id: &str,
    ) -> Result<Service, ApiError> {
        self.get_resource(&Self::service_key(realm_id, hub_id, service_id))
            .await
    }

    pub async fn get_service_with_revision(
        &self,
        realm_id: &str,
        hub_id: &str,
        service_id: &str,
    ) -> Result<(Service, i64), ApiError> {
        self.get_resource_with_revision(&Self::service_key(realm_id, hub_id, service_id))
            .await
    }

    pub async fn list_services(
        &self,
        realm_id: &str,
        hub_id: &str,
    ) -> Result<Vec<Service>, ApiError> {
        self.list_resources(&Self::services_prefix(realm_id, hub_id))
            .await
    }

    pub async fn delete_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service_id: &str,
    ) -> Result<bool, ApiError> {
        self.delete_resource(&Self::service_key(realm_id, hub_id, service_id))
            .await
    }
}
