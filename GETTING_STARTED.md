# chip-in inventory - Getting Started

This guide focuses on the setup and programmatic interaction with the `chip-in inventory` REST API. For a high-level overview of the system architecture, please refer to README.md.

## Prerequisites

*   Docker
*   Docker Compose

## Local Setup

There are two ways to run the application locally:

### Option 1: Using Docker Compose (Recommended for Development)
Starts the inventory and etcd as separate containers. This is useful for development and observing logs from each service.

```bash
docker compose up -d
```

### Option 2: Single Container (Embedded etcd)
Starts both the inventory and etcd inside a single container. Perfect for quick trials or lightweight deployments.

```bash
docker run -d \
  --name chip-in-inventory-embedded \
  -p 3000:3000 \
  -v $(pwd)/conf:/conf \
  -v etcd_data_embedded:/etcd-data \
  -e RUST_LOG=info \
  ghcr.io/srfeo3/chip-in-inventory-embedded-etcd:latest
```

To stop the services started by Docker Compose, run `docker compose down`. For the single container, use `docker stop chip-in-inventory-embedded && docker rm chip-in-inventory-embedded`.

## Development

For active development of the `chip-in-inventory` source code, you can run the `etcd` store in a container and execute the Rust application on your local host. This allows for faster compilation and easier debugging.

### 1. Start the etcd backend
Run a standalone etcd container exposed on localhost:

```bash
docker run -d --rm \
  -p 2379:2379 \
  --name etcd-dev \
  quay.io/coreos/etcd:v3.5.14 \
  /usr/local/bin/etcd \
  --advertise-client-urls http://0.0.0.0:2379 \
  --listen-client-urls http://0.0.0.0:2379
```

### 2. Run the application 

With the backend running, you can now execute the inventory service locally. This is the fastest way to test code changes. 

```bash
# Point to the local etcd and set log level 
ETCD_ENDPOINTS=http://0.0.0.0:2379 RUST_LOG=debug cargo run
```

## Usage

Once the services are running, you can interact with the application through the Web UI or the REST API.

### Web UI
A simple web interface for browsing and basic management.

*   **URL:** `http://localhost:3000`

### REST API
The primary interface for automation and infrastructure-as-code.

*   **Base URL:** `http://localhost:3000/v1`

To verify the API is active, fetch the list of realms:

    curl http://localhost:3000/v1/realms | jq

## Configuration Management

The inventory supports both file-based initialization and dynamic API-based updates.

### 1. File-based Initialization (Startup)
The server can ingest a YAML configuration on startup.
* **Default file**: `conf/config.yaml`
* **Mechanism**: In the `docker-compose.yml` setup, the local `./conf` directory is mounted to `/conf` inside the container. The server processes this file on startup to populate the etcd database.
* **Environment Variable**: The file path is determined by the `CONFIG_FILE` environment variable.

### 2. Dynamic Configuration (REST API)
For runtime updates or programmatic setup, use the REST API. A sample script is provided to demonstrate how to build complex resources (Realms, Zones, Hubs) via the API.

**Applying configuration via script:**
The `scripts/apply_config.sh` script builds a sample realm (`quench`) by sending JSON payloads to the running server.

    ./scripts/apply_config.sh

## Directory Structure

    .
    ├── conf/                 # Startup configuration files (config.yaml)
    ├── scripts/              # Management scripts (apply_config.sh)
    ├── src/                  # Main Rust application source code
    ├── webroot/              # Static assets for the web UI
    ├── Dockerfile            # Packaging the binary into a scratch container
    ├── docker-compose.yml    # Defines the development environment services
    ├── README.md             # System overview and entry point
    └── GETTING_STARTED.md    # This guide
