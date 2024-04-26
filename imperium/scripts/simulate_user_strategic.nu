let work_orders = [
    2100101566,
    2400309103,
    2300004499,
    2100055900,
    2400220518,
    2400206468,
    2100035819,
    2200011153,
    2100036819,
    2100086256,
    2100043948,
    2100066176,
    2400202545,
    2100034503,
    2100069194,
    2100047715,
    2100027677,
    2100063032,
    2100024153,
    2100105109,
    2400325768,
    2200007781,
]


$work_orders | each { |x| imperium strategic scheduling df schedule $x 2024-W49-50 }
^sleep 1
# Replace 'echo "Running command"' with your command
imperium strategic resources df loading 12
# Sleep for 10 seconds using an external shell command
^sleep 1

