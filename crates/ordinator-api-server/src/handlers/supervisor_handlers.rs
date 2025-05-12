// TODO: Move this out
pub async fn handle_supervisor_request<Ss>(
    orchestrator: web::Data<Arc<Mutex<Orchestrator<Ss>>>>,
    supervisor_request: SupervisorRequest,
) -> Result<HttpResponse, actix_web::Error>
where
    Ss: SystemSolutionTrait,
{
    event!(Level::INFO, supervisor_request = ?supervisor_request);
    let supervisor_agent_addrs = match self.agent_registries.get(&supervisor_request.asset) {
        Some(agent_registry) => &agent_registry.supervisor_agent_senders,
        None => {
            return Ok(HttpResponse::BadRequest()
                .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE ASSET"));
        }
    };
    let supervisor_agent_addr = supervisor_agent_addrs
                .iter()
                .find(|(id, _)| id.0 == supervisor_request.supervisor.to_string())
                .expect("This will error at somepoint you will need to handle if you have added additional supervisors")
                .1;

    // This was the reason that we wanted the tokio runtime.
    supervisor_agent_addr
        .sender
        .send(crate::agents::ActorMessage::Actor(
            supervisor_request.supervisor_request_message,
        ))
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response = supervisor_agent_addr
        .receiver
        .recv()
        .map_err(actix_web::error::ErrorInternalServerError)?
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let supervisor_response = SupervisorResponse::new(supervisor_request.asset, response);

    let system_responses = SystemResponses::Supervisor(supervisor_response);
    Ok(HttpResponse::Ok().json(system_responses))
}
