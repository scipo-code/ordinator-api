pub async fn handle_operational_request(
    orchestrator: web::Data<Arc<Mutex<Orchestrator<Ss>>>>,
    operational_request: OperationalRequest,
) -> Result<HttpResponse, actix_web::Error>
{
    let operational_response = match operational_request {
        OperationalRequest::GetIds(asset) => {
            let mut operational_ids_by_asset: Vec<Id> = Vec::new();
            self.agent_registries
                .get(&asset)
                .expect("This error should be handled higher up")
                .operational_agent_senders
                .keys()
                .for_each(|ele| {
                    operational_ids_by_asset.push(ele.clone());
                });

            OperationalResponse::OperationalIds(operational_ids_by_asset)
        }

        // All of this is a bit out of place. I think that the best approach here is to make the
        OperationalRequest::ForOperationalAgent((
            asset,
            operational_id,
            operational_request_message,
        )) => {
            match self
                .agent_registries
                .get(&asset)
                .expect("This error should be handled higher up")
                .get_operational_addr(&operational_id)
            {
                Some(agent_communication) => {
                    agent_communication
                            .sender
                            .send(crate::agents::ActorMessage::Actor(operational_request_message))
                            .context("Could not await the message sending, theard problems are the most likely")
                            .map_err(actix_web::error::ErrorInternalServerError)?;

                    let response = agent_communication
                        .receiver
                        .recv()
                        .map_err(actix_web::error::ErrorInternalServerError)?
                        .map_err(actix_web::error::ErrorInternalServerError)?;

                    OperationalResponse::OperationalState(response)
                }
                None => OperationalResponse::NoOperationalAgentFound(operational_id),
            }
        }
        OperationalRequest::AllOperationalStatus(asset) => {
            let operational_request_status = OperationalStatusRequest::General;
            let operational_request_message =
                OperationalRequestMessage::Status(operational_request_status);
            let mut operational_responses: Vec<OperationalResponseMessage> = vec![];

            let agent_registry_option = self.agent_registries.get(&asset);

            let agent_registry = match agent_registry_option {
                Some(agent_registry) => agent_registry,
                None => {
                    return Ok(HttpResponse::BadRequest()
                        .json("STRATEGIC: STRATEGIC AGENT NOT INITIALIZED FOR THE ASSET"));
                }
            };

            for operational_addr in agent_registry.operational_agent_senders.values() {
                operational_addr
                    .sender
                    .send(crate::agents::ActorMessage::Actor(
                        operational_request_message.clone(),
                    ))
                    .unwrap();
            }

            for operational_addr in agent_registry.operational_agent_senders.values() {
                let response = operational_addr.receiver.recv().unwrap().unwrap();

                operational_responses.push(response);
            }
            OperationalResponse::AllOperationalStatus(operational_responses)
        }
    };
    let system_responses = SystemResponses::Operational(operational_response);
    Ok(HttpResponse::Ok().json(system_responses))
}
