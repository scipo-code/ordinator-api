fish -c "cargo run -p scheduling_system &"

^sleep 105
imperium orchestrator initialize-crew-from-file df imperium/scripts/strategic/resource_configurations/high_resources.toml

^sleep 100
imperium orchestrator initialize-crew-from-file df imperium/scripts/strategic/resource_configurations/base_resources.toml

^sleep 100
imperium orchestrator initialize-crew-from-file df imperium/scripts/strategic/resource_configurations/low_resources.toml

^sleep 60
ps | where name == "scheduling_syst" | kill $in.pid.0 

cd ../generalized-multi-agent-maintenance-scheduling-system/ | just nushell-strategic-data-extract strategic_objective_value_resource_subtraction_plot.tex
