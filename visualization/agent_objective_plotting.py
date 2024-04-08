import pandas as pd
import matplotlib.pyplot as plt
import json
from datetime import datetime
from dotenv import load_dotenv
import os

def parse_log_file(file_path, field_name):
    data = []

    with open(file_path, 'r') as file:
        for line in file:
            try:
                json_obj = json.loads(line)
                objective_value = float(json_obj['fields'][field_name])
                timestamp = json_obj['timestamp']
                data.append({'timestamp': timestamp, field_name: objective_value})
            except json.JSONDecodeError:
                None
                #print(f"Error decoding JSON for line: {line}")
            except KeyError:
                None
                #print(f"Missing expected key in JSON object: {line}")

    return pd.DataFrame(data)

load_dotenv()
today = datetime.now().date()
# Path to your log file
file_path = str(os.getenv("ORDINATOR_LOG_DIR")) + "/ordinator.log." + str(today)

# Parse the log file
strategic_dataframe = parse_log_file(file_path, 'strategic_objective_value')
tactical_dataframe = parse_log_file(file_path, 'tactical_objective_value')


# Convert 'Timestamp' to datetime for plotting
strategic_dataframe['timestamp'] = pd.to_datetime(strategic_dataframe['timestamp'])
tactical_dataframe['timestamp'] = pd.to_datetime(tactical_dataframe['timestamp'])


# Plotting
plt.figure(figsize=(10, 12))


plt.subplot(2, 1, 1)
plt.plot(tactical_dataframe['timestamp'], tactical_dataframe['tactical_objective_value'])
plt.title('Tactical Objective Value Over Time')
plt.xlabel('timestamp')
plt.ylabel('Tactical Objective Value')
plt.xticks(rotation=45)
plt.tight_layout()

plt.subplot(2, 1, 2)
plt.plot(strategic_dataframe['timestamp'], strategic_dataframe['strategic_objective_value'])
plt.title('Strategic Objective Value Over Time')
plt.xlabel('timestamp')
plt.ylabel('Strategic Objective Value')
plt.xticks(rotation=45)
plt.tight_layout()

plt.savefig('visualization/images/objective_value.png')
plt.show()


