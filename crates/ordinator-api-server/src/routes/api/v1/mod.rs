pub fn api_scope() -> actix_web::Scope {
    web::scope("/api")
        .route(
            "/scheduler/export/{asset}",
            web::get().to(scheduler_excel_export),
        )
        .route("/scheduler/assets", web::get().to(scheduler_asset_names))
}
