zellij:
    zellij --layout ordinator.kdl --session ordinator-api

version-bump SEMVER:
    cargo release --no-publish {{SEMVER}}
    
build-windows:
    cross build --target x86_64-pc-windows-gnu --release

build-linux:
    cargo build --release

tr REGEX:
    tail -F logging/logs/ordinator.developer.log | rg {{ REGEX }}
    
call-strategic-inclusion-script:
    #!/usr/bin/env nu
    nu imperium/scripts/strategic/simulate_scheduling_inclusion.nu

call-strategic-exclusion-script:
    #!/usr/bin/env nu
    nu imperium/scripts/strategic/simulate_scheduling_exclusion.nu

call-strategic-resources-addition-script:
    #!/usr/bin/env nu
    nu imperium/scripts/strategic/simulate_resources_addition.nu

call-strategic-resources-subtraction-script:
    #!/usr/bin/env nu
    nu imperium/scripts/strategic/simulate_resources_subtraction.nu

call-strategic-work-order-value-script:
    #!/usr/bin/env nu
    nu imperium/scripts/strategic/simulate_weight_update.nu

list-all-work-orders: 
    #!/usr/bin/env nu
    let work_order_state = imperium status work-orders work-order-state df normal | from json
    $work_order_state | get Orchestrator | get WorkOrderStatus | get Multiple | columns | hx

call-create-all-plot-for-ablns: call-strategic-inclusion-script call-strategic-exclusion-script call-strategic-resources-addition-script call-strategic-resources-subtraction-script call-strategic-work-order-value-script
    echo "All 5 simulation scripts have been called"
