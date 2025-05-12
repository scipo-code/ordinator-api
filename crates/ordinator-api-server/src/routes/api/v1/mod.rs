mod strategic;

use axum::Router;
use strategic::scheduler_nest;

pub async fn api_scope() -> Router
{
    Router::new()
        .nest("scheduler/", scheduler_nest().await)
        .nest("/supervisor", router)
}
// pub fn api_scope() -> actix_web::Scope
// {
//     // Add routes like shown below
//     //
//     web::scope("/api/v1")
//     // .route(
//     //     "/scheduler/export/{asset}",
//     //     web::get().to(scheduler_excel_export),
//     // )
//     // .route("/scheduler/assets", web::get().to(scheduler_asset_names))
// }
//
// ISSUE #131 TODO [ ]
// Replace the `SystemMessages` structure with routers instead.
