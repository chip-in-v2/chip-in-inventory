# chip-in inventory - Getting Started

This guide focuses on the setup and programmatic interaction with the `chip-in inventory` REST API. For a high-level overview of the system architecture, please refer to README.md.

## Prerequisites

*   Docker
*   Docker Compose

## Local Setup

To get the application running locally, you need to have Docker and Docker Compose installed.

1.  **Start the services:**
    This command will pull the pre-built Rust application image from GitHub Container Registry and start it along with the `etcd` server.
    On startup, the application automatically loads the initial configuration from `conf/config.yaml` into the database.

    ```bash
    docker compose up -d
    ```

2.  **Stop the services:**
    To stop and remove the containers, networks, and volumes created by `up`, run:

    ```bash
    docker compose down
    ```

## Development

For faster iteration during development or testing, you can run only the `etcd` backend:

```bash
docker run -d --rm -p 2379:2379 --name etcd-dev quay.io/coreos/etcd:v3.5.14 /usr/local/bin/etcd --advertise-client-urls http://0.0.0.0:2379 --listen-client-urls http://0.0.0.0:2379
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
