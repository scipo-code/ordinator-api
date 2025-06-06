zellij:
    zellij --layout ordinator.kdl --session "ordinator-api"

version-bump SEMVER EXECUTE="":
    #!/usr/bin/env fish
    cargo release --no-publish {{SEMVER}} {{EXECUTE}} &&  rg -o '([0-9]\.[0-9]\.[0-9])' scheduling_system/Cargo.toml > version 

release-on-github VERSION:    
    gh release create {{VERSION}} ./target/release/imperium --title "Release {{VERSION}}" --notes "download the imperium executable with: ```curl -L --output imperium https://github.com/scipo-code/ordinator-api/releases/download/v0.2.2/imperium```"  

build-windows:
    cross build --target x86_64-pc-windows-gnu --release

build-linux:
    cargo build --release

tr REGEX:
    tail -F logging/logs/ordinator.operational.log | rg {{ REGEX }} | jq
    
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

profile-thread TID DURATION:
    #!/usr/bin/env bash
    set -euo pipefail

    usage() {
        echo "Usage: $0 <TID> [<duration_sec>]"
        echo
        echo "TID: THREAD ID to profile"
        echo "duration_sec: Optional. Time to profile. Defaults to 10s if not provided."
        exit 1
    }

    TID={{ TID }} 
    DURATION={{ DURATION }}

    OUTPUT="profiling/out.perf"
    FOLDED="profiling/folded.perf"
    SVG="profiling/flamegraph.svg"

    rm -rf profiling/*

    echo "Recording perf data for TID=$TID for $DURATION seconds..."
    sudo perf record --call-graph dwarf --all-user -F 999 -o "profiling/perf.data" -g --tid "$TID" -- sleep "$DURATION"

    echo "Converting perf.data to out.perf..."
    sudo perf script -i "profiling/perf.data" > "$OUTPUT"

    echo "Converting the out.perf to a folded file"
    stackcollapse-perf.pl "$OUTPUT" > "$FOLDED"

    echo "Generating flame graph out.svg..."
    flamegraph.pl "$FOLDED" > "$SVG"

    echo "Done. Opening svg file"
    firefox "$SVG"
