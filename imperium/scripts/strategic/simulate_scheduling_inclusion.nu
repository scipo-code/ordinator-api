let work_orders = [
    2400372504,
    2100114787,
    2100046479,
    2100045844,
    2400373181,
    2400235609,
    2400286624,
    2100070242,
    2100102626,
    2100048141,
    2100022948,
    2200018148,
    2400220519,
    2100118895,
    2100076856,
    2400360161,
    2100068040,
    2400302678,
    2200019941,
    2400235696,
    2100086073,
    2400360677,
    2400281428,
    2200009247,
    2100073516,
    2200018149,
    2400263657,
    2400373200,
    2200012586,
    2400322307,
    2100086264,
    2100085079,
    2100073336,
    2100040472,
    2300001388,
    2100057173,
    2200009256,
    2200012625,
    2100094703,
    2300004061,
    2400352425,
    2100029794,
    2400355730,
    2100122316,
    2100104299,
    2200009683,
    2400235597,
    2400365539,
    2400325354,
    2100076000,
];

fish -c "cargo build -p scheduling_system --release"
fish -c "cargo run -p scheduling_system --release &"

^sleep 65
imperium strategic scheduling df schedule ...$work_orders 2025-W1-2

^sleep 60
imperium strategic scheduling df schedule ...$work_orders 2025-W3-4

^sleep 60
imperium strategic scheduling df schedule ...$work_orders 2025-W5-6

^sleep 60
imperium strategic scheduling df schedule ...$work_orders 2025-W7-8

^sleep 60
imperium strategic scheduling df schedule ...$work_orders 2025-W9-10

^sleep 60
ps | where name =~ "scheduling_syst" | kill $in.pid.0 --force

cd ../generalized-multi-agent-maintenance-scheduling-system/ | just nushell-strategic-data-extract strategic_objective_value_inclusion_plot.tex
^sleep 10
