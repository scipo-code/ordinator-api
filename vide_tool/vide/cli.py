import sys
import json
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import datetime
import argparse

def read_input():
    # Read all lines from stdin and join them into a single string
    input_data = sys.stdin.read()
    # Convert the JSON string into a Python object
    return json.loads(input_data)

def parse_args():
    parser = argparse.ArgumentParser(description="Illustration tool of Ordinator")
    subparsers = parser.add_subparsers(dest='command', required=True, help='Plotting options for the strategic agent')

    strategic_parser = subparsers.add_parser('strategic', help='Visualizations for the strategic agent')
    tactical_parser = subparsers.add_parser('tactical', help='Visualizations for the tactical agent')

    strategic_parser.add_argument('--offset', type=int, default=0, help='observation offset for the x-axis')
    strategic_parser.add_argument('--title', type=str, required=True, help='graph title')
    strategic_parser.add_argument('--relative', type=bool, required=True, help='controls whether the time on the x-axis is absolute or relative')
    # tactical_parser.add_argument('--input', type=str, required=True, help="JSON input data")
    
    return parser.parse_args()


def main():
    # Read data from stdin

    args = parse_args()

    if not sys.stdin.isatty():
        input_data = sys.stdin.read()
        json_data = json.loads(input_data)
        if args.command == 'strategic':
            strategic_plotting(args, json_data)
        elif args.command == 'tactical':
            tactical_plotting(args, json_data)
    else: 
        println("Data should always be piped into vide")

def strategic_plotting(args, json_data):

    df = pd.DataFrame(json_data)

    # Convert timestamp strings to datetime and strategic objective values to float
    df['timestamp'] = pd.to_datetime(df['timestamp'])
    df['fields_strategic_objective_value'] = df['fields_strategic_objective_value'].astype(float)

    # Plotting
    plt.figure(figsize=(10, 5))
    if args.relative:
        df['rel_time'] = (df['timestamp'] - df['timestamp'].iloc[0]).dt.seconds
        plt.plot(df['rel_time'][args.offset:], np.log(df['fields_strategic_objective_value'][args.offset:]), linestyle='-', antialiased=True)
    else:
        plt.plot(df['timestamp'][args.offset:], np.log(df['fields_strategic_objective_value'][args.offset:]), linestyle='-', antialiased=True)
        
    plt.yscale('linear')
    plt.title(args.title)
    plt.xlabel('Time')
    plt.ylabel('Objective Value')
    plt.grid(True)
    plt.xticks(rotation=45)
    plt.tight_layout()  # Adjust layout to make room for the rotated x-axis labels

    current_time = datetime.datetime.now()
    plt.savefig(f"vide_tool/figures/objective{current_time}.png")
    # Show the plot
    plt.show()
