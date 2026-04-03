/// Data models for the inventory repository.
///
/// Defines structures and logic for Realms, Zones, Hubs, and other
/// resources according to the Chip-in inventory specification.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structure for error responses
#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

/// Structure representing Realm information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Realm {
    pub name: String, // Realm name
    pub title: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    pub cacert: String,
    pub device_id_signing_key: String,
    pub device_id_verification_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_timeout: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub administrators: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<String>,
    pub disabled: bool,
    pub updated_at: DateTime<Utc>,
}

impl Realm {
    pub fn generate_urn(name: &str) -> String {
        format!("urn:chip-in:realm:{}", name)
    }
}

/// Structure used as a request body when creating a new Realm
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRealm {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub cacert: String,
    pub device_id_signing_key: String,
    pub device_id_verification_key: String,
    #[serde(default)]
    pub session_timeout: Option<i64>,
    #[serde(default)]
    pub administrators: Option<Vec<String>>,
    #[serde(default)]
    pub expired_at: Option<String>,
    pub disabled: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a Realm
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRealm {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub cacert: String,
    pub device_id_signing_key: String,
    pub device_id_verification_key: String,
    #[serde(default)]
    pub session_timeout: Option<i64>,
    #[serde(default)]
    pub administrators: Option<Vec<String>>,
    #[serde(default)]
    pub expired_at: Option<String>,
    pub disabled: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Zone information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub name: String, // Zone name
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acme_certificate_provider: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl Zone {
    pub fn generate_urn(realm: &str, name: &str) -> String {
        format!("urn:chip-in:zone:{}:{}", realm, name)
    }
}

/// Structure used as a request body when creating a new Zone
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewZone {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub dns_provider: Option<String>,
    #[serde(default)]
    pub acme_certificate_provider: Option<String>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a Zone
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateZone {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub dns_provider: Option<String>,
    #[serde(default)]
    pub acme_certificate_provider: Option<String>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Subdomain information for storage and full response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Subdomain {
    pub name: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_realm: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub share_cookie: bool,
    // Read-only fields, populated on retrieval
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl Subdomain {
    pub fn generate_urn(realm: &str, zone: &str, name: &str) -> String {
        format!("urn:chip-in:subdomain:{}:{}:{}", realm, zone, name)
    }

    /// Generates FQDN based on subdomain name and zone name
    pub fn generate_fqdn(zone_name: &str, name: &str) -> String {
        if name == "@" {
            zone_name.to_string()
        } else {
            format!("{}.{}", name, zone_name)
        }
    }

    /// Parses a subdomain URN into (realm, zone, name)
    pub fn parse_urn(urn: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = urn.split(':').collect();
        if parts.len() == 6 && parts[0] == "urn" && parts[1] == "chip-in" && parts[2] == "subdomain" {
            Some((parts[3].to_string(), parts[4].to_string(), parts[5].to_string()))
        } else {
            None
        }
    }
}

/// Structure for creating a new Subdomain
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSubdomain {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub realm: Option<String>,
    #[serde(default)]
    pub destination_realm: Option<String>,
    #[serde(default)]
    pub share_cookie: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure for updating a Subdomain. Note: `name` is immutable.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubdomain {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub realm: Option<String>,
    #[serde(default)]
    pub destination_realm: Option<String>,
    #[serde(default)]
    pub share_cookie: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing VirtualHost information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VirtualHost {
    pub name: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    pub subdomain: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_log_recorder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_log_max_value_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_log_format: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl VirtualHost {
    pub fn generate_urn(realm: &str, name: &str) -> String {
        format!("urn:chip-in:virtual-host:{}:{}", realm, name)
    }
}

/// Structure representing VirtualHost information for API responses, including derived fields.
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VirtualHostResponse {
    pub name: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    pub subdomain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_recorder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_max_value_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Structure used as a request body when creating a VirtualHost
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVirtualHost {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub subdomain: String,
    #[serde(default)]
    pub access_log_recorder: Option<String>,
    #[serde(default)]
    pub access_log_max_value_length: Option<i32>,
    #[serde(default)]
    pub access_log_format: Option<serde_json::Value>,
    #[serde(default)]
    pub certificate: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a VirtualHost
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVirtualHost {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub subdomain: String,
    #[serde(default)]
    pub access_log_recorder: Option<String>,
    #[serde(default)]
    pub access_log_max_value_length: Option<i32>,
    #[serde(default)]
    pub access_log_format: Option<serde_json::Value>,
    #[serde(default)]
    pub certificate: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

// --- RoutingChain related structures ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub upstream: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_scope_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Redirect {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReturnStaticText {
    pub content: String,
    pub status: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RequireAuthentication {
    pub auth_scope_name: String,
    pub protected_upstream: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
    pub oidc_authorization_endpoint: String,
    pub oidc_redirect_url: String,
    pub oidc_token_endpoint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oidc_dialect: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetUpstreamRequestHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetDownstreamResponseHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    Proxy(Proxy),
    Redirect(Redirect),
    ReturnStaticText(ReturnStaticText),
    RequireAuthentication(RequireAuthentication),
    SetUpstreamRequestHeader(SetUpstreamRequestHeader),
    SetDownstreamResponseHeader(SetDownstreamResponseHeader),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rule {
    #[serde(rename = "match")]
    pub match_condition: String,
    pub action: Action,
}

/// Structure representing RoutingChain information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoutingChain {
    pub name: String, // RoutingChain name
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Structure used for creating and updating a RoutingChain
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRoutingChain {
    #[serde(default)]
    pub name: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used for updating a RoutingChain
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoutingChain {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Hub information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hub {
    pub name: String, // Hub name
    pub title: String,
    pub fqdn: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    pub server_cert: String,
    pub server_cert_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub attributes: serde_json::Value,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl RoutingChain {
    pub fn generate_urn(realm: &str, name: &str) -> String {
        format!("urn:chip-in:routing-chain:{}:{}", realm, name)
    }
}

/// Structure used as a request body when creating a new Hub
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHub {
    pub name: String,
    pub title: String,
    pub fqdn: String,
    #[serde(default)]
    pub server_address: Option<String>,
    #[serde(default)]
    pub server_port: Option<u16>,
    pub server_cert: String,
    pub server_cert_key: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: serde_json::Value,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Hub {
    pub fn generate_urn(realm: &str, name: &str) -> String {
        format!("urn:chip-in:network:{}:{}", realm, name)
    }
}

/// Structure used as a request body when updating a Hub
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHub {
    pub title: String,
    pub fqdn: String,
    #[serde(default)]
    pub server_address: Option<String>,
    #[serde(default)]
    pub server_port: Option<u16>,
    pub server_cert: String,
    pub server_cert_key: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: serde_json::Value,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Service information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String, // Service name
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub realm: String,
    pub provider: String,
    pub consumers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availability_management: Option<AvailabilityManagement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub singleton: Option<bool>,
    // Read-only fields
    pub hub: String,
    pub urn: String,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl Service {
    pub fn generate_urn(realm: &str, hub: &str, name: &str) -> String {
        format!("urn:chip-in:service:{}:{}:{}", realm, hub, name)
    }
}

/// Structure representing AvailabilityManagement information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvailabilityManagement {
    pub cluster_manager_urn: String,
    pub service_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ondemand_start_on_consumer: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ondemand_start_on_payload: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_timeout: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mount_points: Option<Vec<MountPoint>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MountPoint {
    pub volume_size: i32,
    pub target: String,
}

/// Structure used as a request body when creating a new Service
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewService {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub provider: String,
    pub consumers: Vec<String>,
    #[serde(default)]
    pub availability_management: Option<AvailabilityManagement>,
    #[serde(default)]
    pub singleton: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a Service
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateService {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub provider: String,
    pub consumers: Vec<String>,
    #[serde(default)]
    pub availability_management: Option<AvailabilityManagement>,
    #[serde(default)]
    pub singleton: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}
