// road initial config road module on start up
//
// /home/kai/chip-in-inventory/src/config.rs

use crate::models::{
    Hub, NewHub, NewRealm, NewRoutingChain, NewService, NewSubdomain, NewVirtualHost, NewZone,
    Realm, RoutingChain, Service, Subdomain, VirtualHost, Zone,
};
use crate::repository::EtcdRepository;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

#[derive(Deserialize)]
struct Config {
    realms: Vec<RealmConfig>,
}

#[derive(Deserialize)]
struct RealmConfig {
    #[serde(flatten)]
    base: NewRealm,
    #[serde(default)]
    zones: Vec<ZoneConfig>,
    #[serde(default, rename = "virtualHosts")]
    virtual_hosts: Vec<NewVirtualHost>,
    #[serde(default, rename = "routingChains")]
    routing_chains: Vec<NewRoutingChain>,
    #[serde(default)]
    hubs: Vec<HubConfig>,
}

#[derive(Deserialize)]
struct ZoneConfig {
    #[serde(flatten)]
    base: NewZone,
    #[serde(default)]
    subdomains: Vec<NewSubdomain>,
}

#[derive(Deserialize)]
struct HubConfig {
    #[serde(flatten)]
    base: NewHub,
    #[serde(default)]
    services: Vec<NewService>,
}

pub async fn load_initial_config(repo: &EtcdRepository, config_path: &str) {
    let path = Path::new(&config_path);

    if !path.exists() {
        info!(
            "Config file not found at {:?}, skipping initial population.",
            path
        );
        return;
    }

    info!("Loading initial configuration from {:?}", path);
    let content = match tokio::fs::read_to_string(path).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to read config file: {}", e);
            return;
        }
    };

    let config: Config = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to parse config file: {}", e);
            return;
        }
    };

    for realm_config in config.realms {
        let now = chrono::Utc::now();
        let realm = Realm {
            name: realm_config.base.name.clone(),
            title: realm_config.base.title,
            description: realm_config.base.description,
            urn: Some(format!("urn:chip-in:realm:{}", realm_config.base.name)),
            cacert: realm_config.base.cacert,
            device_id_signing_key: realm_config.base.device_id_signing_key,
            device_id_verification_key: realm_config.base.device_id_verification_key,
            session_timeout: realm_config.base.session_timeout,
            administrators: realm_config.base.administrators,
            expired_at: realm_config.base.expired_at,
            disabled: realm_config.base.disabled,
            created_at: realm_config.base.created_at.unwrap_or(now),
            updated_at: realm_config.base.updated_at.unwrap_or(now),
        };

        if let Err(e) = repo.save_realm(&realm).await {
            tracing::error!("Failed to save realm {}: {}", realm.name, e);
            continue;
        }
        info!("Initialized realm: {}", realm.name);

        // Zones
        for zone_config in realm_config.zones {
            let zone = Zone {
                name: zone_config.base.name.clone(),
                title: zone_config.base.title,
                description: zone_config.base.description,
                dns_provider: zone_config.base.dns_provider,
                acme_certificate_provider: zone_config.base.acme_certificate_provider,
                urn: Some(format!(
                    "urn:chip-in:zone:{}:{}",
                    realm.name, zone_config.base.name
                )),
                realm: Some(format!("urn:chip-in:realm:{}", realm.name)),
                created_at: zone_config.base.created_at.unwrap_or(now),
                updated_at: zone_config.base.updated_at.unwrap_or(now),
            };
            if let Err(e) = repo.save_zone(&realm.name, &zone).await {
                tracing::error!("Failed to save zone {}: {}", zone.name, e);
            }

            // Subdomains
            for sub_config in zone_config.subdomains {
                let fqdn = if sub_config.name == "@" {
                    zone.name.clone()
                } else {
                    format!("{}.{}", sub_config.name, zone.name)
                };
                let subdomain = Subdomain {
                    name: sub_config.name.clone(),
                    title: sub_config.title,
                    description: sub_config.description,
                    realm: sub_config.realm,
                    destination_realm: sub_config.destination_realm,
                    share_cookie: sub_config.share_cookie,
                    fqdn: Some(fqdn),
                    zone: Some(format!("urn:chip-in:zone:{}:{}", realm.name, zone.name)),
                    urn: Some(format!(
                        "urn:chip-in:subdomain:{}:{}:{}",
                        realm.name, zone.name, sub_config.name
                    )),
                    created_at: sub_config.created_at.unwrap_or(now),
                    updated_at: sub_config.updated_at.unwrap_or(now),
                };
                if let Err(e) = repo
                    .save_subdomain(&realm.name, &zone.name, &subdomain)
                    .await
                {
                    tracing::error!("Failed to save subdomain {}: {}", subdomain.name, e);
                }
            }
        }

        // VirtualHosts
        for vhost_config in realm_config.virtual_hosts {
            let vhost = VirtualHost {
                name: vhost_config.name.clone(),
                title: vhost_config.title,
                description: vhost_config.description,
                realm: Some(format!("urn:chip-in:realm:{}", realm.name)),
                urn: Some(format!(
                    "urn:chip-in:virtual-host:{}:{}",
                    realm.name, vhost_config.name
                )),
                subdomain: vhost_config.subdomain,
                access_log_recorder: vhost_config.access_log_recorder,
                access_log_max_value_length: vhost_config.access_log_max_value_length,
                access_log_format: vhost_config.access_log_format,
                certificate: vhost_config.certificate,
                key: vhost_config.key,
                disabled: vhost_config.disabled,
                created_at: vhost_config.created_at.unwrap_or(now),
                updated_at: vhost_config.updated_at.unwrap_or(now),
            };
            if let Err(e) = repo.save_virtual_host(&realm.name, &vhost).await {
                tracing::error!("Failed to save virtual host {}: {}", vhost.name, e);
            }
        }

        // RoutingChains
        for rc_config in realm_config.routing_chains {
            let name = rc_config
                .name
                .clone()
                .unwrap_or_else(|| "default".to_string());
            let rc = RoutingChain {
                name: name.clone(),
                title: rc_config.title,
                description: rc_config.description,
                urn: Some(format!(
                    "urn:chip-in:routing-chain:{}:{}",
                    realm.name, name
                )),
                realm: Some(format!("urn:chip-in:realm:{}", realm.name)),
                rules: rc_config.rules.unwrap_or_default(),
                created_at: rc_config.created_at.unwrap_or(now),
                updated_at: rc_config.updated_at.unwrap_or(now),
            };
            if let Err(e) = repo.save_routing_chain(&realm.name, &rc).await {
                tracing::error!("Failed to save routing chain {}: {}", rc.name, e);
            }
        }

        // Hubs
        for hub_config in realm_config.hubs {
            let hub = Hub {
                name: hub_config.base.name.clone(),
                title: hub_config.base.title,
                fqdn: hub_config.base.fqdn,
                server_address: hub_config.base.server_address,
                server_port: hub_config.base.server_port,
                server_cert: hub_config.base.server_cert,
                server_cert_key: hub_config.base.server_cert_key,
                description: hub_config.base.description,
                realm: Some(format!("urn:chip-in:realm:{}", realm.name)),
                urn: Some(format!(
                    "urn:chip-in:network:{}:{}",
                    realm.name, hub_config.base.name
                )),
                attributes: hub_config.base.attributes,
                created_at: hub_config.base.created_at.unwrap_or(now),
                updated_at: hub_config.base.updated_at.unwrap_or(now),
            };
            if let Err(e) = repo.save_hub(&realm.name, &hub).await {
                tracing::error!("Failed to save hub {}: {}", hub.name, e);
            }

            // Services
            for svc_config in hub_config.services {
                let svc = Service {
                    name: svc_config.name.clone(),
                    title: svc_config.title,
                    description: svc_config.description,
                    realm: format!("urn:chip-in:realm:{}", realm.name),
                    provider: svc_config.provider,
                    consumers: svc_config.consumers,
                    availability_management: svc_config.availability_management,
                    singleton: svc_config.singleton,
                    hub: format!("urn:chip-in:network:{}:{}", realm.name, hub.name),
                    urn: format!(
                        "urn:chip-in:service:{}:{}:{}",
                        realm.name, hub.name, svc_config.name
                    ),
                    created_at: svc_config.created_at.unwrap_or(now),
                    updated_at: svc_config.updated_at.unwrap_or(now),
                };
                if let Err(e) = repo.save_service(&realm.name, &hub.name, &svc).await {
                    tracing::error!("Failed to save service {}: {}", svc.name, e);
                }
            }
        }
    }
}
