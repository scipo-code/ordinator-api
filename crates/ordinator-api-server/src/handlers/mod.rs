// mod orchestrator_handlers;
// pub mod http_to_scheduling_system;
use actix_web::{HttpResponse, web};
use anyhow::Result;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;

// use crate::orchestrator::orchestrator;
use ordinator_scheduling_environment::{
    work_order::{
        WorkOrder, WorkOrderNumber, WorkOrders,
        operation::{Operation, Operations},
    },
    worker_environment::resources::Resources,
};

#[derive(Debug, Deserialize)]
pub struct ApiAsset {
    asset: String,
}

pub async fn get_asset_resources(
    asset: web::Path<ApiAsset>,
) -> Result<HttpResponse, actix_web::Error> {
    #[derive(Debug, Deserialize, Serialize)]
    struct ResourceData {
        pub assets: Vec<AssetResourceData>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct AssetResourceData {
        pub asset: String,
        pub metadata: ResourceMetadata,
        pub data: Vec<ResourceDataPeriod>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct ResourceMetadata {
        pub periods: Vec<Period>,
        pub resources: Vec<Resource>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct Period {
        pub id: String,
        pub label: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct Resource {
        pub id: String,
        pub label: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct ResourceDataPeriod {
        pub periodId: String,
        pub values: serde_json::Value,
    }

    let resource_data = serde_json::from_str::<ResourceData>(
        r#"
            {
            "assets": [
              {
                "asset": "DF",
                "metadata": {
                  "periods": [
                    { "id": "Week1-2", "label": "Week 1-2" },
                    { "id": "Week3-4", "label": "Week 3-4" },
                    { "id": "Week5-6", "label": "Week 5-6" },
                    { "id": "Week7-8", "label": "Week 7-8" }
                  ],
                  "resources": [
                    { "id": "MTN-ELEC", "label": "MTN-ELEC" },
                    { "id": "MTN-INST", "label": "MTN-INST" },
                    { "id": "MTN-MECH", "label": "MTN-MECH" },
                    { "id": "MTN-CRAN", "label": "MTN-CRAN" },
                    { "id": "MTN-ROUS", "label": "MTN-ROUS" }
                  ]
                },
                "data": [
                  {
                    "periodId": "Week1-2",
                    "values": {
                      "MTN-ELEC": 2,
                      "MTN-INST": 2,
                      "MTN-MECH": 2,
                      "MTN-CRAN": 1,
                      "MTN-ROUS": 4
                    }
                  },
                  {
                    "periodId": "Week3-4",
                    "values": {
                      "MTN-ELEC": 3,
                      "MTN-INST": 2,
                      "MTN-MECH": 2,
                      "MTN-CRAN": 1,
                      "MTN-ROUS": 5
                    }
                  },
                  {
                    "periodId": "Week5-6",
                    "values": {
                      "MTN-ELEC": 2,
                      "MTN-INST": 3,
                      "MTN-MECH": 3,
                      "MTN-CRAN": 2,
                      "MTN-ROUS": 4
                    }
                  },
                  {
                    "periodId": "Week7-8",
                    "values": {
                      "MTN-ELEC": 4,
                      "MTN-INST": 3,
                      "MTN-MECH": 2,
                      "MTN-CRAN": 2,
                      "MTN-ROUS": 6
                    }
                  }
                ]
              },
              {
                "asset": "TL",
                "metadata": {
                  "periods": [
                    { "id": "Week1-2", "label": "Week 1-2" },
                    { "id": "Week3-4", "label": "Week 3-4" },
                    { "id": "Week5-6", "label": "Week 5-6" },
                    { "id": "Week7-8", "label": "Week 7-8" }
                  ],
                  "resources": [
                    { "id": "MTN-TELE", "label": "MTN-TELE" },
                    { "id": "MTN-TURB", "label": "MTN-TURB" }
                  ]
                },
                "data": [
                  {
                    "periodId": "Week1-2",
                    "values": {
                      "MTN-TELE": 1,
                      "MTN-TURB": 1
                    }
                  },
                  {
                    "periodId": "Week3-4",
                    "values": {
                      "MTN-TELE": 2,
                      "MTN-TURB": 2
                    }
                  },
                  {
                    "periodId": "Week5-6",
                    "values": {
                      "MTN-TELE": 1,
                      "MTN-TURB": 2
                    }
                  },
                  {
                    "periodId": "Week7-8",
                    "values": {
                      "MTN-TELE": 2,
                      "MTN-TURB": 3
                    }
                  }
                ]
              }
            ]
          }
        "#,
    )
    .unwrap();

    let mut filtered_response = resource_data
        .assets
        .into_iter()
        .filter(|a| a.asset.to_lowercase() == asset.asset.to_lowercase())
        .collect::<Vec<_>>();

    if let Some(first_match) = filtered_response.pop() {
        Ok(HttpResponse::Ok().json(first_match))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

// fn random_resource() -> Resources {
//     let resources = [
//         "MTN-PIPF", "VEN-TURB", "CON-VEN", "MTN-LAGG", "VEN-SCAF", "MTN-ROPE", "VEN-INSP",
//         "INP-SITE", "VEN-INST", "MAINONSH", "DRILLING", "WELLMAIN", "WELLSUPV", "WELLTECH",
//         "CON-ELEC", "CON-INPF", "CON-INST", "CON-LAGG", "CON-NDTI", "CON-SCAF", "CON-PAIN",
//         "CON-RIGG", "CON-ROPE", "CON-WELD", "MTN-ROUS", "MTN-CRAN", "MTN-ELEC", "MTN-INST",
//         "MTN-MECH", "MTN-RIGG", "MTN-SCAF", "MTN-PAIN", "MTN-TELE", "MTN-TURB", "MEDIC",
//         "PRODLABO", "PRODTECH", "MTN-SAT", "VEN-ACCO", "VEN-COMM", "VEN-CRAN", "VEN-ELEC",
//         "VEN-HVAC", "VEN-MECH", "VEN-METE", "VEN-SUBS", "VEN-ROPE", "QAQCELEC", "QAQCMECH",
//         "QAQCPAIN", "PRODCCR", "VEN-FFEQ", "CMP-RIGG", "CMP-SCAF", "CON-NPT",
//     ];

//     let chosen = resources.choose(&mut rand::thread_rng()).unwrap();

//     serde_json::from_str(&format!("\"{}\"", chosen)).unwrap()
// }

// fn create_mock_workorder(id: u64) -> WorkOrder {
//     let wo_number = WorkOrderNumber { 0: id };

//     let res = random_resource();

//     let op =

//     WorkOrder {
//         work_order_number: wo_number,
//         main_work_center: res,
//         operations: op,
//         work_order_analytic: wo_analytics,
//         work_order_dates: wo_dates,
//         work_order_info: wo_info,
//     }

// }

pub async fn scheduler_excel_export(// WARN link to application data
    // orchestrator: web::Data<Arc<Mutex<Orchestrator>>>,
    // WARN url query parameters
    // asset: web::Path<Asset>,
) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().into())
}
