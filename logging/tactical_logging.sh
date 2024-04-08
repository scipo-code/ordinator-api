#!/bin/bash

TODAY=$(date +"%Y-%m-%d")

# Path to your JSON log file
LOG_FILE="$ORDINATOR_PATH/logging/logs/ordinator.log.$TODAY"

# jq filter to select logs where 'target' starts with the specified string
JQ_FILTER='select(.target | startswith("scheduling_system::agents::tactical_agent::"))'

# Function to apply the jq filter to the log file
apply_filter() {
    tail -f "$LOG_FILE" | jq "$JQ_FILTER"
}

# Run the filter function
apply_filter