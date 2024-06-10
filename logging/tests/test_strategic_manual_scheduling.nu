
let date = date now

let data = open logging/logs/ordinator.log.$date

let prev_value = nothing

for entry in $data {
	if $prev_value != $nothing) {
		if ($entry.value > $prev.value) {

			echo "At time $entry.time, the value increased to $entry.value from $prev_value"

		}
	}

	$prev_value = $entry.value
}
