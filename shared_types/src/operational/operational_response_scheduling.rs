use serde::Serialize;

#[derive(Serialize)]
pub enum OperationalSchedulingResponse {
    CalendarEvents(Events),
}


struct Events {
    title: String,
    start_time: 
}
