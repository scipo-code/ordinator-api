pub async fn handle_tactical_request<Ss>(
    orchestrator: web::Data<Arc<Mutex<Orchestrator<Ss>>>>,
    tactical_request: TacticalRequest,
) -> Result<HttpResponse, actix_web::Error>
where
    Ss: SystemSolutionTrait,
{
    let actor_registry_for_asset = match self.agent_registries.get(&tactical_request.asset) {
        Some(agent_registry) => &agent_registry.tactical_agent_sender,
        None => {
            return Ok(HttpResponse::BadRequest()
                .json("TACTICAL: TACTICAL AGENT NOT INITIALIZED FOR THE ASSET"));
        }
    };

    // We should use the
    actor_registry_for_asset
        .sender
        .send(crate::agents::ActorMessage::Actor(
            tactical_request.tactical_request_message,
        ))
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response = actor_registry_for_asset
        .receiver
        .recv()
        .map_err(actix_web::error::ErrorInternalServerError)?
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // We should only have the Message for the particular actor. We should route the
    // message with the `orchestrator`.
    let tactical_response = TacticalResponse::new(tactical_request.asset, response);
    let system_responses = SystemResponses::Tactical(tactical_response);
    Ok(HttpResponse::Ok().json(system_responses))
}
