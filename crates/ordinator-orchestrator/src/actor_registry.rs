use std::collections::HashMap;

use flume::Receiver;
use flume::Sender;
use ordinator_operational_actor::messages::OperationalRequestMessage;
use ordinator_operational_actor::messages::OperationalResponseMessage;
use ordinator_orchestrator_actor_traits::ActorError;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_strategic_actor::messages::StrategicRequestMessage;
use ordinator_strategic_actor::messages::StrategicResponseMessage;
use ordinator_supervisor_actor::messages::SupervisorRequestMessage;
use ordinator_supervisor_actor::messages::SupervisorResponseMessage;
use ordinator_tactical_actor::messages::TacticalRequestMessage;
use ordinator_tactical_actor::messages::TacticalResponseMessage;

pub struct ActorRegistry
{
    pub strategic_agent_sender: Communication<StrategicRequestMessage, StrategicResponseMessage>,
    pub tactical_agent_sender: Communication<TacticalRequestMessage, TacticalResponseMessage>,
    pub supervisor_agent_senders:
        HashMap<Id, Communication<SupervisorRequestMessage, SupervisorResponseMessage>>,
    pub operational_agent_senders:
        HashMap<Id, Communication<OperationalRequestMessage, OperationalResponseMessage>>,

    pub error_channel: (Receiver<ActorError>, Sender<ActorError>),
}

impl ActorRegistry
{
    pub fn get_operational_addr(
        &self,
        operational_id: &String,
    ) -> Option<&Communication<OperationalRequestMessage, OperationalResponseMessage>>
    {
        self.operational_agent_senders
            .iter()
            .find(|(id, _)| &id.0 == operational_id)
            .map(|(_, addr)| addr)
    }

    // This function should be generic over all the different types of messages.
    // So the idea behind this function is that it should take a generic for
    // the interal message, but that the outer message is the same for every
    // agent! This means that it should take like `Status` or something like
    // that
    // FIX
    // Make this generic
    // WARN
    // Making this generic is probably not the best idea.
    // We should  let the `AgentStatus`
    // pub fn recv_all_agents_status(&self) -> Result<AgentStatus> {
    //     let mut supervisor_statai: Vec<SupervisorResponseStatus> = vec![];
    //     let mut operational_statai: Vec<OperationalResponseStatus> = vec![];

    //     let strategic = self.strategic_agent_sender.receiver.recv()??;

    //     let strategic_status = if let StrategicResponseMessage::Status(strategic)
    // = strategic {         strategic
    //     } else {
    //         panic!()
    //     };

    //     let tactical = self.tactical_agent_sender.receiver.recv()??;
    //     let tactical_status = if let TacticalResponseMessage::Status(tactical) =
    // tactical {         tactical
    //     } else {
    //         panic!()
    //     };

    //     for receiver in self.supervisor_agent_senders.iter() {
    //         let supervisor = receiver.1.receiver.recv()??;
    //         if let SupervisorResponseMessage::Status(supervisor) = supervisor {
    //             supervisor_statai.push(supervisor);
    //         } else {
    //             panic!()
    //         }
    //     }
    //     for receiver in self.operational_agent_senders.iter() {
    //         let operational = receiver.1.receiver.recv()??;

    //         if let OperationalResponseMessage::Status(operational) = operational
    // {             operational_statai.push(operational);
    //         } else {
    //             panic!()
    //         }
    //     }

    //     // I am not sure that this is what we want
    //     let agent_status = AgentStatus {
    //         strategic_status,
    //         tactical_status,
    //         supervisor_statai,
    //         operational_statai,
    //     };
    //     Ok(agent_status)
    // }
}
