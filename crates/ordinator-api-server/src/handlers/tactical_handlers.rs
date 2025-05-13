use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::SystemSolutionTrait;
use ordinator_orchestrator::TacticalRequestMessage;
use ordinator_orchestrator::TacticalResponseMessage;
use ordinator_orchestrator::TacticalStatusMessage;

// So each handler should construct a specific message. That is the key point
// here. This function uses the orchestrator to send any kind of message. Which
// way is the correct one here?
//
// ESSAY: What is the best thing to put into the `TacticalRequest`? The
// fundamental issue here is what should be in the URL. I think that we should
// put the data inside of the messages into a JSON but that the handlers should
// only take a single RequestMessage and a corresponding `<Actor>StatusMessage`.
// That means that the handler here should only construct a single message
pub async fn handle_tactical_request<Ss>(
    State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
    Path(asset): Path<Asset>,
) -> Result<Response>
where
    Ss: SystemSolutionTrait,
{
    let message = TacticalRequestMessage::Status(TacticalStatusMessage::General);
    let orchestrator1 = orchestrator.lock().unwrap();
    let actor_registry_for_asset = &orchestrator1
        .actor_registries
        .get(&asset)
        .with_context(|| format!("Asset {} not initialized", &asset))?
        .tactical_agent_sender;

    // We should use the
    // ESSAY: How to handle the string here? I think that the best approach is to
    // avoid the `ActorMessage`, the idea with the enum was that we should provide
    // an interface to the `Actor` that makes it so that only the
    // `ActorMessage::Request` can be chosen. That means that what really has to
    // change is the way that `Communication is implemented`
    actor_registry_for_asset.from_agent(message);

    let response = actor_registry_for_asset.receiver.recv()??;

    // ESSAY:
    // Should you create a handler for each message? I think that is the best
    // approach. The most important thing about the enum structure is that the
    // Actor each know how to handle the request. On the caller side it is less
    // important. There we simply want to construct the correct variant and give
    // the `Communication` that piece of information. I believe that is a crucial
    // insight here.
    // We should only have the Message for the particular actor. We should route the
    // message with the `orchestrator`.
    Ok(Json(response).into_response())
}
