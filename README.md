# Dev Location Service

A Rust-based service for location data management and lookup.
Used as a quick AWS Location Service look up service for local development - when AWS is not suitable.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE.md)

## About

Dev Location Service is a Rust application for managing and querying location data.

It supports multiple backends and can be extended for various use cases.

By default `predefined` back is used which has `data/places.yaml` as a source data and uses some naive heuristics like levenshtein distance or random pick if nothing relevant is found.

Alternative is OpenStreetMap Open API - while more correct it requires working network connection.

## Installation

```sh
# Clone the repository
git clone https://github.com/nazriel/dev-location-service.git
cd dev-location-service

cargo build --release
```

## Usage

```sh
export PORT=8080
export LOCATION_BACKEND=predefined
export PLACES_CONFIG_PATH=./data/places.yaml
cargo run --release
```

The service will start and listen for requests.
See configuration for details.

## Configuration

Service accepts 3 env variables:

- PORT - port to listen on (default: 8080)
- LOCATION_BACKEND - backend to use (default: `predefined`, can be `osm` for OpenStreetMap)
- PLACES_CONFIG_PATH - path to the places configuration file (default: `./data/places.yaml`)

Also you can provide your own list of places in file similiar to `data/places.yaml` in the repository.

## API

The service provides these endpoints:

- `GET /health` - Checks service health
- `GET /info` - Returns service information
- `POST /search` - Searches for places by name. Payload:
  ```json
  {
    "Text": "Warszawa",
    "MaxResults": 1
  }
  ```
- `POST /position` - Searches for places by geographic position. `Position` is `[longitude, latitude]`.
  Payload:
  ```json
  {
    "Position": [21.0122, 52.2297],
    "MaxResults": 1
  }
  ```
- `POST /places/v0/indexes/{index-name}/search/text` - AWS-compatible text search endpoint
  Payload:
  ```json
  {
    "Text": "Warszawa",
    "MaxResults": 1
  }
  ```
- `POST /places/v0/indexes/{index-name}/search/position` - AWS-compatible position search endpoint. It accepts the same `Position` payload as `/position`.

## Contributing

Contributions are welcome! Please open issues or submit pull requests.

## License

This project is licensed under the MIT License. See the [LICENSE.md](LICENSE.md) file for details.

## Contact

For questions or support, open an issue or contact the maintainer.
