using JSON
using DataFrames
using Dates
using Plots
using DotEnv

# Function to parse log file
function parse_log_file(json_data::Vector{Any})
    data = []

    for entry in json_data
        print(entry)
        obj = entry["fields_strategic_objective_value"]
        println(obj)
        strategic_objective_value = parse(Float64, obj)
        timestamp = DateTime(entry["timestamp"])
        push!(data, (Timestamp = timestamp, Strategic_Objective_Value = strategic_objective_value))
    end
    DataFrame(data)
end

input_data = read(stdin, String)

json_data = JSON.parse(input_data)
# Current day
println(typeof(json_data))
df = parse_log_file(json_data)
print(df)
# Plotting
plot(df.Timestamp, df.Strategic_Objective_Value, seriestype = :line, marker = :circle,
     title = "Strategic Objective Value Over Time",
     xlabel = "Timestamp", ylabel = "Strategic Objective Value", legend = false)
xticks = range(minimum(df.Timestamp), stop = maximum(df.Timestamp), length = 10)
plot!(xticks = xticks, xrotation = 45)
