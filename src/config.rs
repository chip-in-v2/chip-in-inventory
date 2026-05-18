/// Configuration loader and initial data population logic.
///
/// Handles reading the YAML configuration file and synchronizing the
/// initial state with the etcd backend during application startup.

use crate::models::{
    Hub, NewHub, NewRealm, NewRoutingChain, NewService, NewSubdomain, NewVirtualHost, NewZone,
    Realm, RoutingChain, Service, Subdomain, VirtualHost, Zone,
};
use crate::repository::EtcdRepository;
use serde::Deserialize;
use std::path::Path;
use tracing::info;
use crate::ApiError;
use futures::future::join_all;

#[derive(Deserialize)]
struct Config {
    realms: Vec<RealmConfig>,
}

fn validate_urn(resource_type: &str, name: &str, yaml_urn: Option<&String>, generated_urn: &str) -> Result<(), ApiError> {
    if let Some(urn) = yaml_urn {
        if urn != generated_urn {
            let msg = format!(
                "URN mismatch for {} '{}': config.yaml has '{}', but generated URN is '{}'.",
                resource_type, name, urn, generated_urn
            );
            return Err(ApiError::BadRequest(msg));
        }
    } else {
        tracing::warn!(
            "URN not found for {} '{}' in config.yaml. Using generated URN: '{}'",
            resource_type,
            name,
            generated_urn
        );
    }
    Ok(())
}

#[derive(Deserialize)]
struct RealmConfig {
    #[serde(flatten)]
    base: NewRealm,
    #[serde(default)]
    zones: Vec<ZoneConfig>,
    #[serde(default, rename = "virtualHosts")]
    virtual_hosts: Vec<VirtualHostConfig>,
    #[serde(default, rename = "routingChains")]
    routing_chains: Vec<RoutingChainConfig>,
    #[serde(default)]
    hubs: Vec<HubConfig>,
    urn: Option<String>,
}

#[derive(Deserialize)]
struct ZoneConfig {
    #[serde(flatten)]
    base: NewZone,
    #[serde(default)]
    subdomains: Vec<SubdomainConfig>,
    urn: Option<String>,
}

#[derive(Deserialize)]
struct SubdomainConfig {
    #[serde(flatten)]
    base: NewSubdomain,
    urn: Option<String>,
}

#[derive(Deserialize)]
struct HubConfig {
    #[serde(flatten)]
    base: NewHub,
    #[serde(default)]
    services: Vec<ServiceConfig>,
    urn: Option<String>,
}

#[derive(Deserialize)]
struct ServiceConfig {
    #[serde(flatten)]
    base: NewService,
    urn: Option<String>,
}

#[derive(Deserialize)]
struct VirtualHostConfig {
    #[serde(flatten)]
    base: NewVirtualHost,
    urn: Option<String>,
}

#[derive(Deserialize)]
struct RoutingChainConfig {
    #[serde(flatten)]
    base: NewRoutingChain,
    urn: Option<String>,
}

pub async fn load_initial_config(repo: &EtcdRepository, config_path: &str) -> Result<(), ApiError> {
    let path = Path::new(&config_path);

    if !path.exists() {
        info!(
            "Config file not found at {:?}, skipping initial population.",
            path
        );
        return Ok(());
    }

    info!("Loading initial configuration from {:?}", path);
    let content = tokio::fs::read_to_string(path).await.map_err(|e| {
        tracing::error!("Failed to read config file: {}", e);
        ApiError::BadRequest(e.to_string())
    })?;

    let config: Config = serde_yaml::from_str(&content).map_err(|e| {
        tracing::error!("Failed to parse config file: {}", e);
        ApiError::BadRequest(e.to_string())
    })?;

    for realm_config in config.realms {
        let now = chrono::Utc::now();
        let realm_urn = Realm::generate_urn(&realm_config.base.name);
        validate_urn(
            "Realm",
            &realm_config.base.name,
            realm_config.urn.as_ref(),
            &realm_urn,
        )?;
        let realm = Realm {
            name: realm_config.base.name.clone(),
            title: realm_config.base.title,
            description: realm_config.base.description,
            urn: Some(realm_urn),
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

        repo.save_realm(&realm).await.map_err(|e| {
            tracing::error!("Failed to save realm {}: {}", realm.name, e);
            e
        })?;
        info!("Initialized realm: {}", realm.name);

        // Zones
        let zone_futures = realm_config.zones.into_iter().map(|zone_config| {
            let repo = repo.clone();
            let realm_name = realm.name.clone();
            async move {
                let zone_urn = Zone::generate_urn(&realm_name, &zone_config.base.name);
                if let Err(e) = validate_urn(
                    "Zone",
                    &zone_config.base.name,
                    zone_config.urn.as_ref(),
                    &zone_urn,
                ) {
                    tracing::error!("Skipping Zone initialization: {}", e);
                    return;
                }

                let now = chrono::Utc::now();
                let zone = Zone {
                    name: zone_config.base.name.clone(),
                    title: zone_config.base.title,
                    description: zone_config.base.description,
                    dns_provider: zone_config.base.dns_provider,
                    acme_certificate_provider: zone_config.base.acme_certificate_provider,
                    urn: Some(zone_urn),
                    realm: Some(Realm::generate_urn(&realm_name)),
                    created_at: zone_config.base.created_at.unwrap_or(now),
                    updated_at: zone_config.base.updated_at.unwrap_or(now),
                };
                if let Err(e) = repo.save_zone(&realm_name, &zone).await {
                    tracing::error!("Failed to save zone {}: {}", zone.name, e);
                }

                // Subdomains
                let sub_futures = zone_config.subdomains.into_iter().map(|sub_config| {
                    let repo = repo.clone();
                    let realm_name = realm_name.clone();
                    let zone_name = zone.name.clone();
                    async move {
                        let sub_urn = Subdomain::generate_urn(&realm_name, &zone_name, &sub_config.base.name);
                        if let Err(e) = validate_urn(
                            "Subdomain",
                            &sub_config.base.name,
                            sub_config.urn.as_ref(),
                            &sub_urn,
                        ) {
                            tracing::error!("Skipping Subdomain initialization: {}", e);
                            return;
                        }

                        let now = chrono::Utc::now();
                        let subdomain = Subdomain {
                            name: sub_config.base.name.clone(),
                            title: sub_config.base.title,
                            description: sub_config.base.description,
                            realm: sub_config.base.realm,
                            destination_realm: sub_config.base.destination_realm,
                            share_cookie: sub_config.base.share_cookie,
                            fqdn: Some(Subdomain::generate_fqdn(&zone_name, &sub_config.base.name)),
                            zone: Some(Zone::generate_urn(&realm_name, &zone_name)),
                            urn: Some(sub_urn),
                            created_at: sub_config.base.created_at.unwrap_or(now),
                            updated_at: sub_config.base.updated_at.unwrap_or(now),
                        };
                        if let Err(e) = repo.save_subdomain(&realm_name, &zone_name, &subdomain).await {
                            tracing::error!("Failed to save subdomain {}: {}", subdomain.name, e);
                        }
                    }
                });
                join_all(sub_futures).await;
            }
        });
        join_all(zone_futures).await;

        // VirtualHosts
        let vhost_futures = realm_config.virtual_hosts.into_iter().map(|vhost_config| {
            let repo = repo.clone();
            let realm_name = realm.name.clone();
            async move {
                let vhost_urn = VirtualHost::generate_urn(&realm_name, &vhost_config.base.name);
                if let Err(e) = validate_urn(
                    "VirtualHost",
                    &vhost_config.base.name,
                    vhost_config.urn.as_ref(),
                    &vhost_urn,
                ) {
                    tracing::error!("Skipping VirtualHost initialization: {}", e);
                    return;
                }

                let now = chrono::Utc::now();
                let vhost = VirtualHost {
                    name: vhost_config.base.name.clone(),
                    title: vhost_config.base.title,
                    description: vhost_config.base.description,
                    realm: Some(Realm::generate_urn(&realm_name)),
                    urn: Some(vhost_urn),
                    subdomain: vhost_config.base.subdomain,
                    access_log_recorder: vhost_config.base.access_log_recorder,
                    access_log_max_value_length: vhost_config.base.access_log_max_value_length,
                    access_log_format: vhost_config.base.access_log_format,
                    certificate: vhost_config.base.certificate,
                    key: vhost_config.base.key,
                    disabled: vhost_config.base.disabled,
                    created_at: vhost_config.base.created_at.unwrap_or(now),
                    updated_at: vhost_config.base.updated_at.unwrap_or(now),
                };
                if let Err(e) = repo.save_virtual_host(&realm_name, &vhost).await {
                    tracing::error!("Failed to save virtual host {}: {}", vhost.name, e);
                }
            }
        });
        join_all(vhost_futures).await;

        // RoutingChains
        let rc_futures = realm_config.routing_chains.into_iter().map(|rc_config| {
            let repo = repo.clone();
            let realm_name = realm.name.clone();
            async move {
                let name = rc_config.base.name.clone().unwrap_or_else(|| "default".to_string());
                let rc_urn = RoutingChain::generate_urn(&realm_name, &name);
                if let Err(e) = validate_urn(
                    "RoutingChain",
                    &name,
                    rc_config.urn.as_ref(),
                    &rc_urn,
                ) {
                    tracing::error!("Skipping RoutingChain initialization: {}", e);
                    return;
                }

                let now = chrono::Utc::now();
                let rc = RoutingChain {
                    name: name.clone(),
                    title: rc_config.base.title,
                    description: rc_config.base.description,
                    urn: Some(rc_urn),
                    realm: Some(Realm::generate_urn(&realm_name)),
                    rules: rc_config.base.rules.unwrap_or_default(),
                    created_at: rc_config.base.created_at.unwrap_or(now),
                    updated_at: rc_config.base.updated_at.unwrap_or(now),
                };
                if let Err(e) = repo.save_routing_chain(&realm_name, &rc).await {
                    tracing::error!("Failed to save routing chain {}: {}", rc.name, e);
                }
            }
        });
        join_all(rc_futures).await;

        // Hubs
        let hub_futures = realm_config.hubs.into_iter().map(|hub_config| {
            let repo = repo.clone();
            let realm_name = realm.name.clone();
            async move {
                let hub_urn = Hub::generate_urn(&realm_name, &hub_config.base.name);
                if let Err(e) = validate_urn(
                    "Hub",
                    &hub_config.base.name,
                    hub_config.urn.as_ref(),
                    &hub_urn,
                ) {
                    tracing::error!("Skipping Hub initialization: {}", e);
                    return;
                }

                let now = chrono::Utc::now();
                let hub = Hub {
                    name: hub_config.base.name.clone(),
                    title: hub_config.base.title,
                    fqdn: hub_config.base.fqdn,
                    server_address: hub_config.base.server_address,
                    server_port: hub_config.base.server_port,
                    server_cert: hub_config.base.server_cert,
                    server_cert_key: hub_config.base.server_cert_key,
                    description: hub_config.base.description,
                    realm: Some(Realm::generate_urn(&realm_name)),
                    urn: Some(hub_urn),
                    attributes: hub_config.base.attributes,
                    created_at: hub_config.base.created_at.unwrap_or(now),
                    updated_at: hub_config.base.updated_at.unwrap_or(now),
                };
                if let Err(e) = repo.save_hub(&realm_name, &hub).await {
                    tracing::error!("Failed to save hub {}: {}", hub.name, e);
                }

                // Services
                let svc_futures = hub_config.services.into_iter().map(|svc_config| {
                    let repo = repo.clone();
                    let realm_name = realm_name.clone();
                    let hub_name = hub.name.clone();
                    async move {
                        let svc_urn = Service::generate_urn(&realm_name, &hub_name, &svc_config.base.name);
                        if let Err(e) = validate_urn(
                            "Service",
                            &svc_config.base.name,
                            svc_config.urn.as_ref(),
                            &svc_urn,
                        ) {
                            tracing::error!("Skipping Service initialization: {}", e);
                            return;
                        }

                        let now = chrono::Utc::now();
                        let svc = Service {
                            name: svc_config.base.name.clone(),
                            title: svc_config.base.title,
                            description: svc_config.base.description,
                            realm: Realm::generate_urn(&realm_name),
                            provider: svc_config.base.provider,
                            consumers: svc_config.base.consumers,
                            availability_management: svc_config.base.availability_management,
                            singleton: svc_config.base.singleton,
                            hub: Hub::generate_urn(&realm_name, &hub_name),
                            urn: svc_urn,
                            created_at: svc_config.base.created_at.unwrap_or(now),
                            updated_at: svc_config.base.updated_at.unwrap_or(now),
                        };
                        if let Err(e) = repo.save_service(&realm_name, &hub_name, &svc).await {
                            tracing::error!("Failed to save service {}: {}", svc.name, e);
                        }
                    }
                });
                join_all(svc_futures).await;
            }
        });
        join_all(hub_futures).await;
    }
    Ok(())
}
