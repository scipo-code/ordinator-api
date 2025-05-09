use actix_web::web;

pub fn api_scope() -> actix_web::Scope
{
    // Add routes like shown below
    //
    web::scope("/api")
    // .route(
    //     "/scheduler/export/{asset}",
    //     web::get().to(scheduler_excel_export),
    // )
    // .route("/scheduler/assets", web::get().to(scheduler_asset_names))
}
