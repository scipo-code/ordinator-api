# Ordinator
Ordinator is a multi-actor scheduling system. The system is based on actors
that each schedule a specific part of the scheduling process in real-time and then communicates 
their solutions to each other and to the users of the system exposed in the form of API endpoints 
provided through ```imperium```.

The real-time responsiveness of the systems means that each agent in the scheduling process will be 
able to react to incoming information from the system whenever and whereever it arrives in the 
scheduling process.

# Example 
Below we see a small example of how to interact with the Ordinator scheduling system. The system is comprised
of two components: The scheduling system itself, and corresponding CLI tool imperium
```rust
cargo run -p scheduling_system  
```
To run the command above you will need to populate the ```temp_scheduling_environment_database``` with 
the mid_*.csv files given by Baptiste.

Running this will run the scheduling system run expose an IP and PORT as specified in the .env file.

```rust
cargo build imperium 
```

This command will create the imperium cli tool, after which it can be found in ./target/debug/imperium. The
corresponding 
```bash 
./target/debug/imperium --help 
```
The help "should" be sufficient to run understand what imperium allows you to do. If this should not be the case please 
submit an issue. The --generator function can be used to generate shell completions for the most common shells. This
will make imperium significantly more ergonomic.




<!-- # Documentation -->

<!-- ## Important High Level Types -->
<!-- This section is to help understand the inner workings of Ordinator, to better allow developers to contribute to --> 
<!-- the code base. -->

<!-- ### SchedulingEnvironment -->
<!-- When manual decisions are made by a user through one of the specific agent instances, the SchedulingEnvironment -->
<!-- will be updated to reflect the latest available knowledge. The other agents of the system then updates --> 
<!-- their states and delivers new solution based the best available knowledge from the scheduling environment. --> 

<!-- The SchedulingEnvironment is implemented as the memory blackboard pattern, this enable all the agents of the system to remain -->
<!-- get the latest state in a scalable way and write to shared memory without corrupting state when writing. The SchedulingEnvironment is initialized -->
<!-- from company data meaning that there is a specific implementation for each data source(s) that has (have) to -->
<!-- implement the following trait: --> 

<!-- ```rust -->
<!-- pub trait SchedulingEnvironmentFactory<DataSource> { -->
    <!-- fn create_scheduling_environment( -->
        <!-- data_source: DataSource, -->
    <!-- ) -> Result<SchedulingEnvironment, SchedulingEnvironmentFactoryError>; -->
<!-- } -->   
<!-- ``` -->

<!-- The SchedulingEnvironment is composed of three types which will briefly be explained here. -->
<!-- #### [WorkOrders](shared_types/src/scheduling_environment/mod.rs); -->

<!-- This types contains all needed information on all work orders (usually abbreviated WO). See the source code type -->
<!-- for additional information. --> 

<!-- #### WorkerEnvironment -->
<!-- The WorkerEnvironment contain all information related to available worker resources. The WorkerEnvironment is --> 
<!-- initialized from a configuration file (for example /imperium/configuration/resources_df.toml). -->

<!-- > Issue: -->
<!-- > - [ ] WorkerEnvironement should be initialized centrally from a single call to the configuration file -->

<!-- #### TimeEnvironment -->
<!-- The TimeEnvironment contains the information needed for specifying the time horizons of the scheduling algorithms implemented --> 
<!-- in the Agents. See type [Period](shared_types/src/scheduling_environment/time_environment/period.rs), [Day](shared_types/src/scheduling_environment/time_environment/day.rs) -->

<!-- ### [Orchestrator](scheduling_system/src/agents/orchestrator.rs) -->
<!-- The Orchestrator is has three main responsibilities -->
<!-- * Create and destroy agents through the [AgentFactory](scheduling_system/src/init/agent_factory.rs) -->
<!-- * Manually change values in the [SchedulingEnvironment](shared_types/src/scheduling_environment/mod.rs) (Dangerous) -->
<!-- * Control logging and tracing setting at runtime [LogHandles](scheduling_system/src/init/logging.rs) -->

<!-- ### [StrategicAgent](scheduling_system/src/agents/strategic_agent/mod.rs) -->
<!-- The StrategicAgent schedules [WorkOrder](shared_types/src/scheduling_environment/work_order/mod.rs)s into weekly or biweekly periods based on a version of the multi-compartment multi-knapsack problem, -->
<!-- which is solved using an implementation of the actor-based large neighborhood search meta-heuristic. -->  

<!-- ### [TacticalAgent](scheduling_system/src/agents/tactical_agent/mod.rs) -->
<!-- The TacticalAgent schedules everything [WorkOrder](shared_types/src/scheduling_environment/work_order/mod.rs)s and their corresponding [Operation](shared_types/src/scheduling_environment/work_order/operation/mod.rs)s into daily time intervals -->
<!-- specifying how many hours that an [Operation](shared_types/src/scheduling_environment/work_order/operation/mod.rs) should be worked on which day by which kind of skill. The [TacticalAgent] implements -->
<!-- a [TacticalAlgorithm](scheduling_system/src/agents/tactical_agent/algorithm/mod.rs) that solve a version of a resource constrained project scheduling problem using an actor-based large neighborhood search -->
<!-- meta-heuristic. --> 

<!-- ### [SupervisorAgent](scheduling_system/src/agents/supervisor_agent/mod.rs) -->
<!-- The [SupervisorAgent](scheduling_system/src/agents/supervisor_agent/mod.rs) can have multiple running instances simutaneously. The SupervisorAgent receives [WorkOrder](shared_types/src/scheduling_environment/work_order/mod.rs)s from -->
<!-- the [TacticalAgent](scheduling_system/src/agents/tactical_agent/mod.rs) and is responsible for distributing them to individual [OperationalAgent](scheduling_system/src/agents/operational_agent/mod.rs)s it does this using --> 
<!-- an iterative combinatorial auction algorithm which solves a version of the assignment problem. -->

<!-- ### [OperationalAgent](scheduling_system/src/agents/operational_agent/mod.rs) -->
<!-- The [OperationalAgent](scheduling_system/src/agents/operational_agent/mod.rs) is the final level of the agent hierarchy. The [OperationalAgent](scheduling_system/src/agents/operational_agent/mod.rs) implements an actor-based large neighborhood search -->
<!-- meta-heuristic --> 

<!-- ### [Messages](shared_types/src/lib.rs) -->
<!-- To allow for efficient and effective communication between different parts of the system --> 

<!-- #### [SystemMessages](shared_types/src/lib.rs) -->
<!-- The [SystemMessages](shared_types/src/lib.rs) is an enum containing all the messages that interact with the [scheduling_system](scheduling_system/src/main.rs). The enum (so far) has 6 different variants -->
<!-- meaning that there are 6 different ways of interacting with the system. -->  

<!-- ```rust -->
<!-- pub enum SystemMessages { -->
    <!-- Orchestrator(OrchestratorRequest), -->
    <!-- Strategic(StrategicRequest), -->
    <!-- Tactical(TacticalRequest), -->
    <!-- Supervisor(SupervisorRequest), -->
    <!-- Operational(OperationalRequest), -->
    <!-- Sap, -->
<!-- } -->
<!-- ``` -->
<!-- For further explanations see the Request types themselves -->

<!-- #### [SystemResponses](shared_types/src/lib.rs) -->
<!-- The [SystemResponses](shared_types/src/lib.rs) are simply the possible responses that the [SystemMessages](shared_types/src/lib.rs) can provide. The [SystemResponses](shared_types/src/lib.rs) were -->
<!-- primarily created to gain strong types to perform JSON serialization on and for making the API significantly easier to maintain. --> 

<!-- ```rust -->
<!-- pub enum SystemResponses { -->
    <!-- Orchestrator(OrchestratorResponse), -->
    <!-- Strategic(StrategicResponse), -->
    <!-- Tactical(TacticalResponse), -->
    <!-- Supervisor(SupervisorResponse), -->
    <!-- Operational(OperationalResponse), -->
    <!-- Export, -->
    <!-- Sap, -->
<!-- } -->
<!-- ``` -->
<!-- #### [StateLink](scheduling_system/src/agents/mod.rs) -->
<!-- This is a fundamental message of the system as it contain all the ways that agent should communication with each other in what circumstances. That -->
<!-- means that this types handles business logic and complex state management. DUE NOT CHANGE ANYTHING that is related to the [StateLink](scheduling_system/src/agents/mod.rs) unless you --> 
<!-- know both what you are changing programmatically and its implications in the domain. -->

<!-- #### [SetAddr](scheduling_system/src/agents/mod.rs) -->
<!-- This is a simply Message type use to pass around [Addr<impl Actor>] (channel addresses) in the system. [SetAddr](scheduling_system/src/agents/mod.rs) allows agents to discover each other -->
<!-- and communicate. The Message is most frequently used under the initialization of Agents. --> 

<!-- #### [StopMessage](shared_types/src/lib.rs) -->
<!-- This is a simple message to stop an agent. It is needed as Agent run in perpetuity. -->

<!-- #### [ScheduleIteration](scheduling_system/src/agents/mod.rs) -->
<!-- This is a loop back message telling itself to run a new iteration of its main scheudling loop. Ideally this functionality should not be implemented as --> 
<!-- a Message type, but it eases the message implementation significantly as the [ScheduleIteration](scheduling_system/src/agents/mod.rs) message is put on top of an Agent's message queue meaning -->
<!-- that any messages received during an scheduling iteration will be handled before the Agent is allow to continue optimizing. -->

<!-- #### [UpdateWorkOrder](scheduling_system/agents/mod.rs) -->
<!-- This is a stray Message, it should be part of the [OrchestratorRequest](shared_types/src/orchestrator/mod.rs) Message instead. -->
<!-- > Issue: UpdateWorkOrder -->
<!-- >  - [ ] These kind of messages general should fall within the same category of message that change the SchedulingEnvironment, meaning as RequestMessages for the Orchestrator -->

<!-- #### [SolutionExportMessage](shared_types/src/lib.rs) -->
<!-- This is a message that the user sends to a specific agent manually telling it to provide its current solution in a human-readable format for the end user. -->
<!-- Each [Agent] should implement this so that the user gets a static solution based on the Agent matching him, ideally for printing or analysis etc. -->

<!-- #### [TestRequest](scheduling_system/src/agents/strategic_agent/mod.rs) -->
<!-- All Agents implement this Message and it triggers a testing procedure of the given Agent's current state to verify that nothing is out of the ordinary. -->

<!-- #### [OperationSolution](scheduling_system/agents/tactical_agents/tactical_algorithm.rs) -->
<!-- This is another stray Message, it should be refactor under the [StateLink](scheduling_system/src/agents/mod.rs) Message as it is related to how the the [SupervisorAgent](scheduling_system/src/agents/supervisor_agent/mod.rs) handles -->
<!-- and interprets the schedule/solution coming from the [TacticalAgent](scheduling_system/src/agents/tactical_agent/mod.rs). --> 

<!-- #### [StatusMessage](shared_types/src/lib.rs) -->
<!-- Another stary Message. Each Agent should implement a Handler<StatusMessage> but it should be part of the [SystemMessages](shared_types/src/lib.rs) on the Request side -->
<!-- and the return value/result should be given by the [SystemResponses](shared_types/src/lib.rs) Message -->

<!-- ## [Imperium](imperium/src/main.rs) --> 
<!-- Imperium is a command line tool to interact with the Ordinator scheduling system. It contains all the API (in the form of HTTP messages) -->
<!-- that users need to get and modify their schedules in real-time. This API is specified completely in the [SystemMessages](shared_types/src/lib.rs) and [SystemResponses](shared_types/src/lib.rs). -->
<!-- This is not ideal for a future stable deployment, but it does mean that as long as [Imperium](imperium/src/main.rs) and [Ordinator](scheduling_system/src/main.rs) are compiled together in the -->
<!-- workspace that we will have static type guarantees on the HTTPs API that are communicated between them. (NON-TRIVIAL!) --> 

<!-- ## Tracing --> 
<!-- Tracing is a crucial aspect of understand the workings of the code as it is highly parallel. The log level can be set dynamically using --> 
<!-- Imperium. (Setting it to Level::TRACING, will overload your system due to extremely high number of writes to the hard drive) -->

<!-- # Profiling and benchmarking -->

<!-- ## Profiling --> 
<!-- Profiling is done throught the tracing.rs and tracing-flame.rs packages. -->  

<!-- ``` -->
<!-- #[instrument] --> 
<!-- fn fun(par: Par) { -->
    <!-- // Do some calculation -->
<!-- } -->
<!-- ``` -->
<!-- This can lead to serious performance issues if the `par` argument is a large and/or nested type, as the --> 
<!-- instrument macro also applies tracing/logging to the function arguments. In that case one should --> 
<!-- use `#[instrument(skip(par))]` on the function definition. -->

<!-- ## Benchmarking -->
<!-- Individual functions and methods can be benchmarked using criterion.rs. Benchmarking should not be required --> 
<!-- unless some bottleneck is discovered that needs to be handled. Determining bottlenecks in the code is best --> 
<!-- done with Profiling using a flamegraph. -->


# Error handling
Error handling is used extensively to test the validity of the system. That means that
Results are used for business logic invariants as well. To make the system standardized
colors are used to standardize the understanding of the colors.

* Time related: Green
* Job related: Yellow
* Resources related: Blue
* Objective: Purple

Every error at runtime should ideally produce a single bug fix on every iteration. If this
is not upheld it means that the error was created in a wrong way.
