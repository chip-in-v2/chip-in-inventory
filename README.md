# chip-in inventory

Component of the SPN (Service Provider Network) infrastructure: a secure, virtualized distribution network for enterprise applications. `chip-in inventory` provides central configuration management and distribution.

## Overview
`chip-in inventory` serves as the etcd-backed control plane for the SPN system. It stores and distributes authoritative configuration data to `spnhub` and `spngw`.

## Key Roles
- **Configuration Repository**: Centralized management of network topology and service definitions.
- **Policy Distribution**: Provides real-time configuration updates to infrastructure components.
- **Note**: Only `spnhub` and `spngw` require access to this service; `spnagent` does not communicate with the inventory directly.

## Configuration (Environment Variables)
- `ETCD_ENDPOINTS`: List of ETCD cluster endpoints for data persistence.
- `CONFIG_FILE`: Path to the inventory data file loaded at startup.
- `RUST_LOG`: Log level (e.g., `info`, `debug`).

## Public Interface
- **HTTP (REST/WebUI)**: Listening on `0.0.0.0:3000`.

## Deployment
- **Binary**: Static `musl` binaries.
- **Container**: Lightweight `scratch`-based images.

## Documentation
The API implements the OpenAPI specification for inventory management defined at https://github.com/chip-in-v2/docusaurus/tree/main/root/openapi/inventory.

For detailed setup, development, and API usage instructions, please refer to GETTING_STARTED.md.