cd $env.FILE_PWD

imperium strategic resources df load-capacity-file ./resource_configurations/low_resources.toml

^sleep 60
imperium strategic resources df load-capacity-file ./resource_configurations/base_resources.toml

^sleep 60
imperium strategic resources df load-capacity-file ./resource_configurations/high_resources.toml

