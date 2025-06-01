# Build stage
FROM rust:1.87.0-alpine3.22 AS builder
WORKDIR /app
COPY . .
RUN apk add --no-cache build-base openssl-dev openssl-libs-static pkgconfig
RUN cargo install --path .
RUN cargo build --release

# Runtime stage
FROM alpine:3.22.0 AS runtime
WORKDIR /app

RUN apk add --no-cache curl

ENV LOCATION_BACKEND=predefined
ENV PLACES_CONFIG_PATH=/app/data/places.yaml

HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 CMD [ "curl", "-f", "http://localhost:8080/health" ]

COPY --from=builder /app/target/release/dev-location-service /app/dev-location-service
COPY --from=builder /app/data ./data

ENTRYPOINT ["/app/dev-location-service"]
