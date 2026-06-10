# CHDRMS backend

## Dev setup

```
pushd backend/database/ && sqlx migrate run && popd
cargo run --package chdrms_server
```
