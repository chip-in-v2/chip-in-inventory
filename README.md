# Chip-in Inventory

This project provides an inventory management server for spn infrastructure system. It offers a RESTful API and a simple web UI for managing inventory, using etcd as the backend data store.
The API implements the OpenAPI specification for inventory management defined at https://github.com/chip-in-v2/docusaurus/tree/main/root/openapi/inventory.

The technology stack includes Rust for the programming language and etcd for the backend database/store.

## Features

- **RESTful API**: For programmatic inventory management.
- **Web UI**: For easy browsing, creation, updating, and deletion of inventory items.
- **etcd Backend**: A reliable distributed key-value store for inventory data.
- **Data Import/Export**: Scripts to import from and export to YAML files.

## Prerequisites

*   Docker
*   Docker Compose

## Quick Start

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

Access the web UI in your browser to manage the inventory. The UI allows you to browse, create, update, and delete inventory objects.

*   **URL:** `http://localhost:3000`

### REST API

The API server is available at the following base URL. You can interact with it using tools like `curl` or Postman.

*   **Base URL:** `http://localhost:3000/v1`

To run a quick API test, you can fetch the list of realms:
```bash
curl http://localhost:3000/v1/realms
```

### Importing and Exporting Data

You can manually import or export inventory data using the provided Python scripts. These scripts can be run directly on your host machine.

**1. Prerequisites**

First, ensure you have Python installed, then install the required libraries using pip:
```bash
pip install pyyaml requests
```

**Import from a YAML file:**
```bash
python ./scripts/import_inventory.py input.yaml
```

**Export from a YAML file:**
```bash
python ./scripts/export_inventory.py > output.yaml
```

## Example Rust Client

The `/examples/rust_client` directory contains a sample command-line application that demonstrates how to interact with the `chip-in-inventory` API.

It shows how to:
1.  Fetch all resources from the API and build a complete, hierarchical in-memory representation of the inventory.
2.  Use the in-memory data to perform a specific task, such as finding the correct routing `Action` for a given FQDN and request path.

To run the client:
```bash
cd examples/rust_client
cargo run
```

## Directory Structure

```
.
├── data/                 # Sample data for the importer
├── examples/             # Example clients and usage
├── scripts/              # Scripts used by services (e.g., importer)
├── src/                  # Main Rust application source code
├── webroot/              # Static assets for the web UI
├── Dockerfile            # Packaging the binary into a scratch container
├── docker-compose.yml    # Defines the development environment services
└── README.md             # This file
```
