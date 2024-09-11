import os
from datetime import datetime, timedelta

# Set the variables
log_folder_names = ["aws-logs7"]
val_index = 0

# Define the function to parse and filter logs
def filter_logs_by_time(log_content, time_interval=timedelta(hours=1)):
    log_lines = log_content.splitlines()
    if not log_lines:
        return ""

    # Parse the timestamp of the last log entry with a valid timestamp
    for i in range(len(log_lines) - 1, -1, -1):
        try:
            last_log_time = datetime.fromisoformat(log_lines[i].split(' ')[0][:-1])
            break
        except ValueError:
            continue
    else:
        return ""

    # Calculate the cutoff time
    cutoff_time = last_log_time - time_interval

    # Filter logs
    filtered_logs = []
    for line in log_lines:
        parts = line.split(' ')
        if len(parts) > 0:
            try:
                log_time = datetime.fromisoformat(parts[0][:-1])
                if log_time >= cutoff_time:
                    filtered_logs.append(line)
            except ValueError:
                continue

    return '\n'.join(filtered_logs)

# Process each log folder
for log_folder_name in log_folder_names:
    log_file_path = os.path.join(os.getcwd(), log_folder_name, f"client-{val_index}.log")

    # Read the file content as a string
    with open(log_file_path, 'r') as file:
        lines = file.read()

    # Replace double newlines to make sure each log stays on one line.
    lines = lines.replace("INFO \n\nAdvanced to block", "INFO Advanced to block")
    lines = lines.replace("\n\nAdvanced to block", "Advanced to block")
    lines = lines.replace("INFO \n\nCommitting a subdag", "INFO Committing a subdag")

    # Filter logs to keep only the last one hour of entries
    filtered_logs = filter_logs_by_time(lines)

    # Store as a new log file
    new_log_file_name = f"prepared_logs_{val_index}.log"
    new_log_file_folder = os.path.join(os.getcwd(), log_folder_name)

    # Create the folder if it does not exist
    if not os.path.exists(new_log_file_folder):
        os.makedirs(new_log_file_folder)
    
    new_log_file_path = os.path.join(new_log_file_folder, new_log_file_name)

    # Write the filtered logs to the new file
    with open(new_log_file_path, 'w') as file:
        file.write(filtered_logs)