# Security Policy

## Supported status

Sky Time Series is an **engineering beta**. CI verifies formatting, compilation, Clippy, tests, a generated dependency-lock audit, release build, Docker build, and non-root image execution. Production infrastructure is not verified by this repository and no production-support SLA is offered.

## Current boundaries

The service validates series key length, finite numeric values, and timestamp-range ordering. It does not implement authentication, authorization, TLS termination, request rate limiting, persistence, encryption at rest, retention enforcement, replication, or multi-tenant isolation.

Operators must provide network access control, TLS, authentication, resource limits, observability, durable-storage policy, backup/restore, and abuse prevention if the service is exposed beyond a trusted development environment.

## Reporting

Please report suspected vulnerabilities privately through GitHub's security reporting mechanisms when available. Do not include credentials, private telemetry, customer data, or exploit payloads in public issues.
