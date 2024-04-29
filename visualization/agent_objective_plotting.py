import sys
import json
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

def read_input():
    # Read all lines from stdin and join them into a single string
    input_data = sys.stdin.read()
    # Convert the JSON string into a Python object
    return json.loads(input_data)

def main():
    # Read data from stdin
    data = read_input()

    # Convert the data into a DataFrame
    df = pd.DataFrame(data)

    # Convert timestamp strings to datetime and strategic objective values to float
    df['timestamp'] = pd.to_datetime(df['timestamp'])
    df['fields_strategic_objective_value'] = df['fields_strategic_objective_value'].astype(float)

    # Plotting
    plt.figure(figsize=(10, 5))
    plt.plot(df['timestamp'][110:], np.log(df['fields_strategic_objective_value'][110:]), linestyle='-')
    plt.yscale('linear')
    plt.title('Objective value over time')
    plt.xlabel('Time')
    plt.ylabel('Objective Value')
    plt.grid(True)
    plt.xticks(rotation=45)
    plt.tight_layout()  # Adjust layout to make room for the rotated x-axis labels

    plt.savefig("visualization/objective.png")
    # Show the plot
    plt.show()

if __name__ == "__main__":
    main()
