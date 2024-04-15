using JSON
using DataFrames
using Dates
using Plots

# Function to parse log file
function parse_log_file(file_path)
    data = []
    open(file_path, "r") do file
        for line in eachline(file)
            try
                json_obj = JSON.parse(line)
                tactical_objective_value = parse(Float64, json_obj["fields"]["tactical_objective_value"])
                timestamp = DateTime(json_obj["timestamp"])
                push!(data, (Timestamp = timestamp, Tactical_Objective_Value = tactical_objective_value))
            catch e
                if isa(e, JSON.ParserError)
                    println("Error decoding JSON for line: $line")
                elseif isa(e, KeyError)
                    println("Missing expected key in JSON object: $line")
                end
            end
        end
    end
    DataFrame(data)
end

# Current day
today = Dates.day(now())

# Path to your log file
ENV["ORDINATOR_LOG_DIR"] = "/path/to/logs" # Replace with actual path or load from .env
file_path = ENV["ORDINATOR_LOG_DIR"] * "/ordinator.log.$today"

# Parse the log file
df = parse_log_file(file_path)

# Plotting
plot(df.Timestamp, df.Tactical_Objective_Value, seriestype = :line, marker = :circle,
     title = "Tactical Objective Value Over Time",
     xlabel = "Timestamp", ylabel = "Tactical Objective Value", legend = false)
xticks = range(minimum(df.Timestamp), stop = maximum(df.Timestamp), length = 10)
plot!(xticks = xticks, xrotation = 45)