use std::collections::HashMap;

use shared_types::agents::operational::OperationalRequestMessage;
use shared_types::agents::operational::OperationalResponseMessage;
use shared_types::agents::strategic::StrategicRequestMessage;
use shared_types::agents::strategic::StrategicResponseMessage;
use shared_types::agents::supervisor::SupervisorRequestMessage;
use shared_types::agents::supervisor::SupervisorResponseMessage;
use shared_types::agents::tactical::TacticalRequestMessage;
use shared_types::agents::tactical::TacticalResponseMessage;
use shared_types::scheduling_environment::worker_environment::resources::Id;

use crate::agents::ActorMessage;

pub struct ActorRegistry {
    pub strategic_agent_sender:
        Communication<ActorMessage<StrategicRequestMessage>, StrategicResponseMessage>,
    pub tactical_agent_sender:
        Communication<ActorMessage<TacticalRequestMessage>, TacticalResponseMessage>,
    pub supervisor_agent_senders: HashMap<
        Id,
        Communication<ActorMessage<SupervisorRequestMessage>, SupervisorResponseMessage>,
    >,
    pub operational_agent_senders: HashMap<
        Id,
        Communication<ActorMessage<OperationalRequestMessage>, OperationalResponseMessage>,
    >,
}

impl ActorRegistry {
    pub fn get_operational_addr(
        &self,
        operational_id: &String,
    ) -> Option<&Communication<ActorMessage<OperationalRequestMessage>, OperationalResponseMessage>>
    {
        let option_id = self
            .operational_agent_senders
            .iter()
            .find(|(id, _)| &id.0 == operational_id)
            .map(|(_, addr)| addr);
        option_id
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
    pub fn recv_all_agents_status(&self) -> Result<AgentStatus> {
        let mut supervisor_statai: Vec<SupervisorResponseStatus> = vec![];
        let mut operational_statai: Vec<OperationalResponseStatus> = vec![];

        let strategic = self.strategic_agent_sender.receiver.recv()??;

        let strategic_status = if let StrategicResponseMessage::Status(strategic) = strategic {
            strategic
        } else {
            panic!()
        };

        let tactical = self.tactical_agent_sender.receiver.recv()??;
        let tactical_status = if let TacticalResponseMessage::Status(tactical) = tactical {
            tactical
        } else {
            panic!()
        };

        for receiver in self.supervisor_agent_senders.iter() {
            let supervisor = receiver.1.receiver.recv()??;
            if let SupervisorResponseMessage::Status(supervisor) = supervisor {
                supervisor_statai.push(supervisor);
            } else {
                panic!()
            }
        }
        for receiver in self.operational_agent_senders.iter() {
            let operational = receiver.1.receiver.recv()??;

            if let OperationalResponseMessage::Status(operational) = operational {
                operational_statai.push(operational);
            } else {
                panic!()
            }
        }

        let agent_status = AgentStatus {
            strategic_status,
            tactical_status,
            supervisor_statai,
            operational_statai,
        };
        Ok(agent_status)
    }
}
