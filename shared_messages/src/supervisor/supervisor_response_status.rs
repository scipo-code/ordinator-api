use crate::models::worker_environment::resources::{Id, MainResources};

pub struct SupervisorResponseStatus {
    id: Id,
    main_work_center: MainResources,
}
