# Campus Hire Dashboard and Rental Management System

(Name WIP)

## Environment Variables
More examples found in the [example configuration](config.example.toml).

| Variable | Default | Description |
|---|---|---|
| `CHDRMS__HOST` | `::` | Address to serve HTTP on. |
| `CHDRMS__PORT` | `3000` | Port to serve HTTP on. |
| `CHDRMS__CONFIGURATION_PATH` | `config.toml` | Configuration file location. |
| `CHDRMS__SECRET_KEY_PATH` | `.secretkey ` | Secret key file location. |
| `CHDRMS__UI_DIRECTORY` | `dist` | Static file server directory. |
| `DATABASE_URL` | | PostgreSQL connection URL. |
| `ENVIRONMENT` | `DEVELOPMENT` | Current running environment. |
