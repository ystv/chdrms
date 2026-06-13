fn main() {
    let (_, api) = chdrms_server::routes::routes();
    let json = api.to_pretty_json().expect("failed to serialise schema");
    std::fs::write(
        std::env::args_os()
            .nth(1)
            .expect("Usage: export_openapi <path>"),
        json,
    )
    .expect("failed to write schema");
}
