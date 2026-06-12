# Guide for contributors

## Dependencies

To work on CHDRMS, you will need a Rust toolchain, alongwith nodejs+yarn. If you use NixOS (or the nix package manager), this repo includes a nix flake with a dev shell, along with a direnv configuration to load this whenever you open the project.

## Backend

The backend depends on PostgreSQL 18 and an OpenID Connect provider, both of which can be started using docker/podman compose. A configuration file is provided for using this OIDC server (`config.dev.toml`).

The default username and password for the included test user is `test`.
