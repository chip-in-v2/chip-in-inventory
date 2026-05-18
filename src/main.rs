/// Repository server for the SPN infrastructure system.
///
/// Provides a RESTful API and Web UI to manage inventory information,
/// using etcd as the backend data store.
/// The data models for the repository are defined at:
///   https://github.com/chip-in-v2/docusaurus/tree/main/root/openapi/inventory

mod models;
mod repository;
mod config;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use models::{
    ErrorResponse, Hub, NewHub, NewRealm, NewRoutingChain, NewService, NewSubdomain,
    NewVirtualHost, NewZone, Realm, RoutingChain, Service, Subdomain, UpdateHub, UpdateRealm,
    UpdateRoutingChain, UpdateService, UpdateSubdomain, UpdateVirtualHost, UpdateZone, VirtualHost,
    VirtualHostResponse, Zone,
};
use repository::EtcdRepository;
use std::{env, net::SocketAddr};
use thiserror::Error;
use tracing::info;
type AppState = EtcdRepository;

/// Validates ID to prevent path traversal.
fn validate_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains(':') {
        return Err(ApiError::BadRequest(format!("Invalid ID format: {}", id)));
    }
    Ok(())
}

/// Web UI (index.html, script.js, style.css)
async fn index_handler() -> impl IntoResponse {
    Html(include_str!("../webroot/index.html"))
}
async fn webui() -> impl IntoResponse {
    // script.js
    Response::builder()
        .header(header::CONTENT_TYPE, "application/javascript;charset=utf-8")
        .body(include_str!("../webroot/script.js").to_owned())
        .unwrap()
}
async fn webui2() -> impl IntoResponse {
    // style.css
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css;charset=utf-8")
        .body(include_str!("../webroot/style.css").to_owned())
        .unwrap()
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Connect to etcd
    let etcd_endpoints_str =
        env::var("ETCD_ENDPOINTS").unwrap_or_else(|_| "http://127.0.0.1:2379".to_string());
    let etcd_endpoints: Vec<&str> = etcd_endpoints_str.split(',').collect();
    let config_path =
        env::var("CONFIG_FILE").unwrap_or_else(|_| "conf/config.yaml".to_string());

    let repository = EtcdRepository::new(&etcd_endpoints)
        .await
        .expect("Failed to connect to etcd");

    config::load_initial_config(&repository, &config_path)
        .await
        .expect("Failed to load initial configuration");

    let ui_routes = Router::new()
        // Web UI
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/script.js", get(webui))
        .route("/style.css", get(webui2));

    let api_routes = Router::new()
        .route("/realms", get(list_realms).post(create_realm))
        .route(
            "/realms/{realm_id}",
            get(get_realm).put(update_realm).delete(delete_realm),
        )
        .route("/realms/{realm_id}/zones", get(list_zones).post(create_zone))
        .route(
            "/realms/{realm_id}/zones/{zone_id}",
            get(get_zone).put(update_zone).delete(delete_zone),
        )
        .route(
            "/realms/{realm_id}/zones/{zone_id}/subdomains",
            get(list_subdomains).post(create_subdomain),
        )
        .route(
            "/realms/{realm_id}/zones/{zone_id}/subdomains/{subdomain_id}",
            get(get_subdomain)
                .put(update_subdomain)
                .delete(delete_subdomain),
        )
        .route(
            "/realms/{realm_id}/virtual-hosts",
            get(list_virtual_hosts).post(create_virtual_host),
        )
        .route(
            "/realms/{realm_id}/virtual-hosts/{virtual_host_id}",
            get(get_virtual_host)
                .put(update_virtual_host)
                .delete(delete_virtual_host),
        )
        .route(
            "/realms/{realm_id}/routing-chains",
            get(list_routing_chains).post(create_routing_chain),
        )
        .route(
            "/realms/{realm_id}/routing-chains/{routing_chain_id}",
            get(get_routing_chain)
                .put(update_routing_chain)
                .delete(delete_routing_chain),
        )
        .route("/realms/{realm_id}/hubs", get(list_hubs).post(create_hub))
        .route(
            "/realms/{realm_id}/hubs/{hub_id}",
            get(get_hub).put(update_hub).delete(delete_hub),
        )
        .route(
            "/realms/{realm_id}/hubs/{hub_id}/services",
            get(list_services).post(create_service),
        )
        .route(
            "/realms/{realm_id}/hubs/{hub_id}/services/{service_id}",
            get(get_service).put(update_service).delete(delete_service),
        );

    // Add /v1 prefix to api_routes and merge with ui_routes
    let app = Router::new()
        .merge(ui_routes)
        .nest("/v1", api_routes)
        .with_state(repository);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}

// Error handling
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("etcd client error: {0}")]
    Etcd(#[from] etcd_client::Error),
    #[error("json serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("resource not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("resource already exists: {0}")]
    Conflict(String),
    #[error("parent resource not found: {0}")]
    ParentNotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Etcd(e) => {
                tracing::error!("etcd client error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Datastore error".to_string())
            },
            ApiError::Json(e) => {
                tracing::error!("JSON error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON processing error: {}", e))
            },
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::ParentNotFound(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        (status, Json(ErrorResponse { message })).into_response()
    }
}

// --- Service Handlers ---

/// Helper to populate read-only fields for a Service
fn populate_service_fields(service: &mut Service, realm_id: &str, hub_id: &str) {
    service.urn = Service::generate_urn(realm_id, hub_id, &service.name);
    service.hub = Hub::generate_urn(realm_id, hub_id);
    service.realm = Realm::generate_urn(realm_id);
}
// --- Realm Handlers ---

// GET /realms
async fn list_realms(State(repo): State<AppState>) -> Result<Json<Vec<Realm>>, ApiError> {
    let realms = repo.list_realms().await?;
    Ok(Json(realms))
}

// POST /realms
async fn create_realm(
    State(repo): State<AppState>,
    Json(payload): Json<NewRealm>,
) -> Result<(StatusCode, Json<Realm>), ApiError> {
    validate_id(&payload.name)?;

    let now = chrono::Utc::now();
    let realm = Realm {
        name: payload.name.clone(),
        description: payload.description,
        title: payload.title,
        urn: Some(Realm::generate_urn(&payload.name)),
        cacert: payload.cacert,
        device_id_signing_key: payload.device_id_signing_key,
        device_id_verification_key: payload.device_id_verification_key,
        session_timeout: payload.session_timeout,
        administrators: payload.administrators,
        expired_at: payload.expired_at,
        disabled: payload.disabled,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    repo.save_realm(&realm).await?;
    Ok((StatusCode::CREATED, Json(realm)))
}

// GET /realms/:realm_id
async fn get_realm(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
) -> Result<Json<Realm>, ApiError> {
    validate_id(&realm_id)?;
    repo.get_realm(&realm_id).await
        .map(Json)
}

// PUT /realms/:realm_id
async fn update_realm(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
    Json(payload): Json<UpdateRealm>,
) -> Result<Json<Realm>, ApiError> {
    validate_id(&realm_id)?;
    let now = chrono::Utc::now();
    let realm = Realm {
        name: realm_id.clone(),
        description: payload.description,
        title: payload.title,
        urn: Some(Realm::generate_urn(&realm_id)),
        device_id_signing_key: payload.device_id_signing_key,
        device_id_verification_key: payload.device_id_verification_key,
        cacert: payload.cacert,
        session_timeout: payload.session_timeout,
        administrators: payload.administrators,
        expired_at: payload.expired_at,
        disabled: payload.disabled,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    repo.save_realm(&realm).await?;
    Ok(Json(realm))
}

// DELETE /realms/:realm_id
async fn delete_realm(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    validate_id(&realm_id)?;
    let deleted = repo.delete_realm(&realm_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

// --- Service Handlers ---

// GET /realms/:realm_id/hubs/:hub_id/services
async fn list_services(
    State(repo): State<AppState>,
    Path((realm_id, hub_id)): Path<(String, String)>,
) -> Result<Json<Vec<Service>>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    let services = repo.list_services(&realm_id, &hub_id).await?;
    let mut populated_services = services;
    for service in &mut populated_services {
        populate_service_fields(service, &realm_id, &hub_id);
    }
    Ok(Json(populated_services))
}

// POST /realms/:realm_id/hubs/:hub_id/services
async fn create_service(
    State(repo): State<AppState>,
    Path((realm_id, hub_id)): Path<(String, String)>,
    Json(payload): Json<NewService>,
) -> Result<(StatusCode, Json<Service>), ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    validate_id(&payload.name)?;

    let service_name = payload.name;
    let now = chrono::Utc::now();
    let mut service = Service {
        name: service_name.clone(),
        title: payload.title,
        description: payload.description,
        realm: String::new(),
        provider: payload.provider,
        consumers: payload.consumers,
        availability_management: payload.availability_management,
        singleton: payload.singleton,
        hub: String::new(),
        urn: String::new(),
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_service_fields(&mut service, &realm_id, &hub_id);
    repo.save_service(&realm_id, &hub_id, &service).await?;
    Ok((StatusCode::CREATED, Json(service)))
}

// GET /realms/:realm_id/hubs/:hub_id/services/:service_id
async fn get_service(
    State(repo): State<AppState>,
    Path((realm_id, hub_id, service_id)): Path<(String, String, String)>,
) -> Result<Json<Service>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    validate_id(&service_id)?;
    let mut service = repo.get_service(&realm_id, &hub_id, &service_id).await?;

    populate_service_fields(&mut service, &realm_id, &hub_id);
    Ok(Json(service))
}

// PUT /realms/:realm_id/hubs/:hub_id/services/:service_id
async fn update_service(
    State(repo): State<AppState>,
    Path((realm_id, hub_id, service_id)): Path<(String, String, String)>,
    Json(payload): Json<UpdateService>,
) -> Result<Json<Service>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    validate_id(&service_id)?;
    let now = chrono::Utc::now();
    let mut service = Service {
        name: service_id.clone(),
        title: payload.title,
        description: payload.description,
        realm: String::new(),
        provider: payload.provider,
        consumers: payload.consumers,
        availability_management: payload.availability_management,
        singleton: payload.singleton,
        hub: String::new(),
        urn: String::new(),
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_service_fields(&mut service, &realm_id, &hub_id);
    repo.save_service(&realm_id, &hub_id, &service).await?;

    Ok(Json(service))
}

// DELETE /realms/:realm_id/hubs/:hub_id/services/:service_id
async fn delete_service(
    State(repo): State<AppState>,
    Path((realm_id, hub_id, service_id)): Path<(String, String, String)>,
) -> Result<StatusCode, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    validate_id(&service_id)?;
    let deleted = repo
        .delete_service(&realm_id, &hub_id, &service_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

// --- Hub Handlers ---

// GET /realms/:realm_id/hubs
async fn list_hubs(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
) -> Result<Json<Vec<Hub>>, ApiError> {
    validate_id(&realm_id)?;
    let mut hubs = repo.list_hubs(&realm_id).await?;
    for hub in &mut hubs {
        populate_hub_fields(hub, &realm_id);
    }
    Ok(Json(hubs))
}

// POST /realms/:realm_id/hubs
async fn create_hub(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
    Json(payload): Json<NewHub>,
) -> Result<(StatusCode, Json<Hub>), ApiError> {
    validate_id(&realm_id)?;
    validate_id(&payload.name)?;

    let now = chrono::Utc::now();
    let mut hub = Hub {
        name: payload.name,
        description: payload.description,
        title: payload.title,
        fqdn: payload.fqdn,
        server_address: payload.server_address,
        server_port: payload.server_port,
        server_cert: payload.server_cert,
        server_cert_key: payload.server_cert_key,
        realm: None,
        urn: None,
        attributes: payload.attributes,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_hub_fields(&mut hub, &realm_id);
    repo.save_hub(&realm_id, &hub).await?;

    Ok((StatusCode::CREATED, Json(hub)))
}

// GET /realms/:realm_id/hubs/:hub_id
async fn get_hub(
    State(repo): State<AppState>,
    Path((realm_id, hub_id)): Path<(String, String)>,
) -> Result<Json<Hub>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    let mut hub = repo.get_hub(&realm_id, &hub_id).await?;

    populate_hub_fields(&mut hub, &realm_id);
    Ok(Json(hub))
}

// PUT /realms/:realm_id/hubs/:hub_id
async fn update_hub(
    State(repo): State<AppState>,
    Path((realm_id, hub_id)): Path<(String, String)>,
    Json(payload): Json<UpdateHub>,
) -> Result<Json<Hub>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    let now = chrono::Utc::now();
    let mut hub = Hub {
        name: hub_id.clone(),
        title: payload.title,
        fqdn: payload.fqdn,
        description: payload.description,
        server_address: payload.server_address,
        server_port: payload.server_port,
        server_cert: payload.server_cert,
        server_cert_key: payload.server_cert_key,
        realm: None,
        urn: None,
        attributes: payload.attributes,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_hub_fields(&mut hub, &realm_id);
    repo.save_hub(&realm_id, &hub).await?;

    Ok(Json(hub))
}

// DELETE /realms/:realm_id/hubs/:hub_id
async fn delete_hub(
    State(repo): State<AppState>,
    Path((realm_id, hub_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&hub_id)?;
    let deleted = repo.delete_hub(&realm_id, &hub_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Helper to populate read-only fields for a Hub
fn populate_hub_fields(hub: &mut Hub, realm_id: &str) {
    hub.urn = Some(Hub::generate_urn(realm_id, &hub.name));
    hub.realm = Some(Realm::generate_urn(realm_id));
}

/// Helper to populate read-only fields for a RoutingChain
fn populate_routing_chain_fields(rchain: &mut RoutingChain, realm_id: &str) {
    rchain.urn = Some(RoutingChain::generate_urn(realm_id, &rchain.name));
    rchain.realm = Some(Realm::generate_urn(realm_id));
}

// --- RoutingChain Handlers ---

// GET /realms/:realm_id/routing-chains
async fn list_routing_chains(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
) -> Result<Json<Vec<RoutingChain>>, ApiError> {
    validate_id(&realm_id)?;
    let mut rchains = repo.list_routing_chains(&realm_id).await?;
    for rchain in &mut rchains {
        populate_routing_chain_fields(rchain, &realm_id);
    }
    Ok(Json(rchains))
}

// POST /realms/:realm_id/routing-chains
async fn create_routing_chain(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
    Json(payload): Json<NewRoutingChain>,
) -> Result<(StatusCode, Json<RoutingChain>), ApiError> {
    validate_id(&realm_id)?;
    let name = payload.name.unwrap_or_else(|| "default".to_string());
    validate_id(&name)?;

    let now = chrono::Utc::now();
    let mut rchain = RoutingChain {
        name,
        title: payload.title,
        description: payload.description,
        urn: None,
        realm: None,
        rules: payload.rules.unwrap_or_default(),
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_routing_chain_fields(&mut rchain, &realm_id);
    repo.save_routing_chain(&realm_id, &rchain).await?;

    Ok((StatusCode::CREATED, Json(rchain)))
}

// GET /realms/:realm_id/routing-chains/:routing_chain_id
async fn get_routing_chain(
    State(repo): State<AppState>,
    Path((realm_id, routing_chain_id)): Path<(String, String)>,
) -> Result<Json<RoutingChain>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&routing_chain_id)?;
    let mut rchain = repo.get_routing_chain(&realm_id).await?;

    if rchain.name != routing_chain_id {
        return Err(ApiError::NotFound);
    }

    populate_routing_chain_fields(&mut rchain, &realm_id);
    Ok(Json(rchain))
}

// PUT /realms/:realm_id/routing-chains/:routing_chain_id
async fn update_routing_chain(
    State(repo): State<AppState>,
    Path((realm_id, routing_chain_id)): Path<(String, String)>,
    Json(payload): Json<UpdateRoutingChain>,
) -> Result<Json<RoutingChain>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&routing_chain_id)?;
    let now = chrono::Utc::now();
    let mut rchain = RoutingChain {
        name: routing_chain_id.clone(),
        title: payload.title,
        description: payload.description,
        urn: None,
        realm: None,
        rules: payload.rules.unwrap_or_default(),
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_routing_chain_fields(&mut rchain, &realm_id);
    repo.save_routing_chain(&realm_id, &rchain).await?;

    Ok(Json(rchain))
}

// DELETE /realms/:realm_id/routing-chains/:routing_chain_id
async fn delete_routing_chain(
    State(repo): State<AppState>,
    Path((realm_id, routing_chain_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&routing_chain_id)?;
    let deleted = repo.delete_routing_chain(&realm_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

// --- VirtualHost Handlers ---

// GET /realms/:realm_id/virtual-hosts
async fn list_virtual_hosts(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
) -> Result<Json<Vec<VirtualHostResponse>>, ApiError> {
    validate_id(&realm_id)?;
    let vhosts = repo.list_virtual_hosts(&realm_id).await?;

    let mut response_vhosts = Vec::new();
    for vhost in vhosts {
        let fqdn = resolve_vhost_fqdn(&repo, &vhost.subdomain, &vhost.name).await?;
        response_vhosts.push(vhost.into_response(fqdn));
    }

    Ok(Json(response_vhosts))
}

/// Helper to resolve VirtualHost FQDN from subdomain URN and vhost name
async fn resolve_vhost_fqdn(
    repo: &EtcdRepository,
    subdomain_urn: &str,
    _vhost_name: &str,
) -> Result<Option<String>, ApiError> {
    if let Some((realm_name, zone_name, subdomain_name)) = Subdomain::parse_urn(subdomain_urn) {
        if let Ok(subdomain) = repo.get_subdomain(&realm_name, &zone_name, &subdomain_name).await {
            return Ok(subdomain.fqdn);
        }
    }
    Ok(None)
}

/// Helper to populate read-only fields for a Subdomain
fn populate_subdomain_fields(subdomain: &mut Subdomain, realm_id: &str, zone_id: &str) {
    subdomain.urn = Some(Subdomain::generate_urn(realm_id, zone_id, &subdomain.name));
    subdomain.realm = Some(Realm::generate_urn(realm_id));
    subdomain.zone = Some(Zone::generate_urn(realm_id, zone_id));
    subdomain.fqdn = Some(Subdomain::generate_fqdn(zone_id, &subdomain.name));
}

/// Helper to populate read-only fields for a Zone
fn populate_zone_fields(zone: &mut Zone, realm_id: &str) {
    zone.urn = Some(Zone::generate_urn(realm_id, &zone.name));
    zone.realm = Some(Realm::generate_urn(realm_id));
}

impl VirtualHost {
    fn into_response(self, fqdn: Option<String>) -> VirtualHostResponse {
        VirtualHostResponse {
            name: self.name,
            title: self.title,
            description: self.description,
            realm: self.realm,
            urn: self.urn,
            fqdn,
            subdomain: self.subdomain,
            access_log_recorder: self.access_log_recorder,
            access_log_max_value_length: self.access_log_max_value_length,
            access_log_format: self.access_log_format,
            certificate: self.certificate,
            key: self.key,
            disabled: self.disabled,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// POST /realms/:realm_id/virtual-hosts
async fn create_virtual_host(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
    Json(payload): Json<NewVirtualHost>,
) -> Result<(StatusCode, Json<VirtualHostResponse>), ApiError> {
    validate_id(&realm_id)?;
    validate_id(&payload.name)?;

    let now = chrono::Utc::now();
    let vhost_name = payload.name.clone();
    let vhost = VirtualHost {
        name: vhost_name.clone(), // No change, but for clarity
        description: payload.description,
        title: payload.title,
        realm: Some(Realm::generate_urn(&realm_id)),
        urn: Some(VirtualHost::generate_urn(&realm_id, &vhost_name)),
        subdomain: payload.subdomain,
        access_log_recorder: payload.access_log_recorder,
        access_log_max_value_length: payload.access_log_max_value_length,
        access_log_format: payload.access_log_format,
        certificate: payload.certificate,
        key: payload.key,
        disabled: payload.disabled,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    repo.save_virtual_host(&realm_id, &vhost).await?;

    let fqdn = resolve_vhost_fqdn(&repo, &vhost.subdomain, &vhost.name).await?;

    Ok((StatusCode::CREATED, Json(vhost.into_response(fqdn))))
}

// GET /realms/:realm_id/virtual-hosts/:virtual_host_id
async fn get_virtual_host(
    State(repo): State<AppState>,
    Path((realm_id, virtual_host_id)): Path<(String, String)>,
) -> Result<Json<VirtualHostResponse>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&virtual_host_id)?;
    let vhost = repo.get_virtual_host(&realm_id, &virtual_host_id).await?;

    let fqdn = resolve_vhost_fqdn(&repo, &vhost.subdomain, &vhost.name).await?;

    Ok(Json(vhost.into_response(fqdn)))
}

// PUT /realms/:realm_id/virtual-hosts/:virtual_host_id
async fn update_virtual_host(
    State(repo): State<AppState>,
    Path((realm_id, virtual_host_id)): Path<(String, String)>,
    Json(payload): Json<UpdateVirtualHost>,
) -> Result<Json<VirtualHostResponse>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&virtual_host_id)?;
    let now = chrono::Utc::now();
    let vhost = VirtualHost {
        name: virtual_host_id.clone(),
        title: payload.title,
        description: payload.description,
        realm: Some(Realm::generate_urn(&realm_id)),
        urn: Some(VirtualHost::generate_urn(&realm_id, &virtual_host_id)),
        subdomain: payload.subdomain,
        access_log_recorder: payload.access_log_recorder,
        access_log_max_value_length: payload.access_log_max_value_length,
        access_log_format: payload.access_log_format,
        certificate: payload.certificate,
        key: payload.key,
        disabled: payload.disabled,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    repo.save_virtual_host(&realm_id, &vhost).await?;

    let fqdn = resolve_vhost_fqdn(&repo, &vhost.subdomain, &vhost.name).await?;

    Ok(Json(vhost.into_response(fqdn)))
}

// DELETE /realms/:realm_id/virtual-hosts/:virtual_host_id
async fn delete_virtual_host(
    State(repo): State<AppState>,
    Path((realm_id, virtual_host_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&virtual_host_id)?;
    let deleted = repo
        .delete_virtual_host(&realm_id, &virtual_host_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

// --- Subdomain Handlers ---

// GET /realms/:realm_id/zones/:zone_id/subdomains
async fn list_subdomains(
    State(repo): State<AppState>,
    Path((realm_id, zone_id)): Path<(String, String)>,
) -> Result<Json<Vec<Subdomain>>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    let mut subdomains = repo.list_subdomains(&realm_id, &zone_id).await?;

    for sub in &mut subdomains {
        populate_subdomain_fields(sub, &realm_id, &zone_id);
    }

    Ok(Json(subdomains))
}

// POST /realms/:realm_id/zones/:zone_id/subdomains
async fn create_subdomain(
    State(repo): State<AppState>,
    Path((realm_id, zone_id)): Path<(String, String)>,
    Json(payload): Json<NewSubdomain>,
) -> Result<(StatusCode, Json<Subdomain>), ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    validate_id(&payload.name)?;

    let now = chrono::Utc::now();
    let mut subdomain = Subdomain {
        name: payload.name,
        title: payload.title,
        description: payload.description,
        realm: payload.realm,
        destination_realm: payload.destination_realm,
        share_cookie: payload.share_cookie,
        fqdn: None,
        zone: None,
        urn: None,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_subdomain_fields(&mut subdomain, &realm_id, &zone_id);
    repo.save_subdomain(&realm_id, &zone_id, &subdomain).await?;

    Ok((StatusCode::CREATED, Json(subdomain)))
}

// GET /realms/:realm_id/zones/:zone_id/subdomains/:subdomain_id
async fn get_subdomain(
    State(repo): State<AppState>,
    Path((realm_id, zone_id, subdomain_id)): Path<(String, String, String)>,
) -> Result<Json<Subdomain>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    validate_id(&subdomain_id)?;
    let mut subdomain = repo.get_subdomain(&realm_id, &zone_id, &subdomain_id).await?;

    populate_subdomain_fields(&mut subdomain, &realm_id, &zone_id);
    Ok(Json(subdomain))
}

// PUT /realms/:realm_id/zones/:zone_id/subdomains/:subdomain_id
async fn update_subdomain(
    State(repo): State<AppState>,
    Path((realm_id, zone_id, subdomain_id)): Path<(String, String, String)>,
    Json(payload): Json<UpdateSubdomain>,
) -> Result<Json<Subdomain>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    validate_id(&subdomain_id)?;
    let now = chrono::Utc::now();
    let mut subdomain = Subdomain {
        name: subdomain_id.clone(),
        title: payload.title,
        description: payload.description,
        realm: payload.realm,
        destination_realm: payload.destination_realm,
        share_cookie: payload.share_cookie,
        fqdn: None,
        zone: None,
        urn: None,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_subdomain_fields(&mut subdomain, &realm_id, &zone_id);
    repo.save_subdomain(&realm_id, &zone_id, &subdomain).await?;

    Ok(Json(subdomain))
}

// DELETE /realms/:realm_id/zones/:zone_id/subdomains/:subdomain_id
async fn delete_subdomain(
    State(repo): State<AppState>,
    Path((realm_id, zone_id, subdomain_id)): Path<(String, String, String)>,
) -> Result<StatusCode, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    validate_id(&subdomain_id)?;
    let deleted = repo
        .delete_subdomain(&realm_id, &zone_id, &subdomain_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

// --- Zone Handlers ---

// GET /realms/:realm_id/zones
async fn list_zones(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
) -> Result<Json<Vec<Zone>>, ApiError> {
    validate_id(&realm_id)?;
    let mut zones = repo.list_zones(&realm_id).await?;
    for zone in &mut zones {
        populate_zone_fields(zone, &realm_id);
    }
    Ok(Json(zones))
}

// POST /realms/:realm_id/zones
async fn create_zone(
    State(repo): State<AppState>,
    Path(realm_id): Path<String>,
    Json(payload): Json<NewZone>,
) -> Result<(StatusCode, Json<Zone>), ApiError> {
    validate_id(&realm_id)?;
    validate_id(&payload.name)?;

    let now = chrono::Utc::now();
    let mut zone = Zone {
        name: payload.name.clone(),
        title: payload.title,
        description: payload.description,
        dns_provider: payload.dns_provider,
        acme_certificate_provider: payload.acme_certificate_provider,
        urn: None,
        realm: None,
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_zone_fields(&mut zone, &realm_id);
    repo.save_zone(&realm_id, &zone).await?;

    Ok((StatusCode::CREATED, Json(zone)))
}

// GET /realms/:realm_id/zones/:zone_id
async fn get_zone(
    State(repo): State<AppState>,
    Path((realm_id, zone_id)): Path<(String, String)>,
) -> Result<Json<Zone>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    let mut zone = repo.get_zone(&realm_id, &zone_id).await?;

    populate_zone_fields(&mut zone, &realm_id);
    Ok(Json(zone))
}

// PUT /realms/:realm_id/zones/:zone_id
async fn update_zone(
    State(repo): State<AppState>,
    Path((realm_id, zone_id)): Path<(String, String)>,
    Json(payload): Json<UpdateZone>,
) -> Result<Json<Zone>, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    let now = chrono::Utc::now();
    let mut zone = Zone {
        name: zone_id.clone(),
        title: payload.title,
        description: payload.description,
        dns_provider: payload.dns_provider,
        acme_certificate_provider: payload.acme_certificate_provider,
        urn: None,
        realm: Some(Realm::generate_urn(&realm_id)),
        created_at: payload.created_at.unwrap_or(now),
        updated_at: payload.updated_at.unwrap_or(now),
    };

    populate_zone_fields(&mut zone, &realm_id);
    repo.save_zone(&realm_id, &zone).await?;

    Ok(Json(zone))
}

// DELETE /realms/:realm_id/zones/:zone_id
async fn delete_zone(
    State(repo): State<AppState>,
    Path((realm_id, zone_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    validate_id(&realm_id)?;
    validate_id(&zone_id)?;
    let deleted = repo
        .delete_zone(&realm_id, &zone_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
