pub mod baptiste_csv_reader;
pub mod baptiste_csv_reader_merges;
pub mod excel;

use shared_types::scheduling_environment::SchedulingEnvironment;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulingEnvironmentFactoryError {
    #[error("error while creating SchedulingEnvironment from excel file")]
    ExcelError(#[from] calamine::Error),
}

pub trait SchedulingEnvironmentFactory<DataSource> {
    fn create_scheduling_environment(
        data_source: DataSource,
    ) -> Result<SchedulingEnvironment, SchedulingEnvironmentFactoryError>;
}
