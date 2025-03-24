use actix_web::web;

use crate::handlers::get_asset_resources;

pub fn api_scope() -> actix_web::Scope {
    // Add routes like shown below
    //
    web::scope("/api/v1").route("/{asset}/resources", web::get().to(get_asset_resources))
    // .route(
    //     "/scheduler/export/{asset}",
    //     web::get().to(scheduler_excel_export),
    // )
    // .route("/scheduler/assets", web::get().to(scheduler_asset_names))
}
