let work_orders = [
    2400325768,
    2400284299,
    2400263357,
    2100080290,
    2100100118,
    2400322775,
    2400309483,
    2400329996,
    2200012832,
    2200012642,
    2400235610,
    2100053893,
    2100023869,
    2200007083,
    2100059454,
    2100107512,
    2400284226,
    2200009862,
    2100022645,
    2400281089,
    2400325729,
    2400316058,
    2400235040,
    2100096905,
    2400294027,
]


./imperium/scripts/initialize.sh

^sleep 180
$work_orders | each { |x| imperium strategic scheduling df schedule $x 2024-W49-50 }

^sleep 180
$work_orders | each { |x| imperium strategic scheduling df schedule $x 2024-W39-40 }

^sleep 180
$work_orders | each { |x| imperium strategic scheduling df schedule $x 2024-W29-30 }

