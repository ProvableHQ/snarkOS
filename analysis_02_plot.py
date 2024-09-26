import os
import pandas as pd
import matplotlib.pyplot as plt
import copy
import numpy as np

# Set the variables
num_val = 15
val_index = 3
log_file_name = f"prepared_logs_{val_index}.log"
log_file_path = os.path.join(os.getcwd(), "aws-logs9", log_file_name)

# Load the log file
with open(log_file_path, 'r') as file:
    lines = file.readlines()

# Split each line into a timestamp and a message, filtering out invalid lines
data = []
for line in lines:
    parts = line.strip().split('  INFO ', 1)
    if len(parts) == 2:
        try:
            timestamp = pd.to_datetime(parts[0], format='%Y-%m-%dT%H:%M:%S.%f%z')
            data.append(parts)
        except ValueError:
            continue

# Convert to DataFrame
df = pd.DataFrame(data, columns=['Timestamp', 'Message'])
df['Timestamp'] = pd.to_datetime(df['Timestamp'])

# Filter specific events
events = [
    "SYNCPROFILING", 
    "Advanced to block",
]

event_df = df[df['Message'].str.contains('|'.join(events), na=False)]

# for debugging
# store the event_df in a csv file
# event_df.to_csv(f'event_df_val{val_index}.csv', index=False)


fig, ax = plt.subplots(figsize=(10, 7))
# legend
ax.legend()
used_labels = {}

# starttime is time of first "Time to find sync peers" log
time_to_find_sync_peers = event_df[event_df['Message'].str.contains('Time to find sync peers', na=False)]
start_time = time_to_find_sync_peers.iloc[0]['Timestamp']

# find "SYNCPROFILING Time to construct request" logs
time_to_construct_requests = event_df[event_df['Message'].str.contains('SYNCPROFILING Time to construct request', na=False)]
# iterate
offset = event_df.iloc[0].name
for i, row in time_to_construct_requests.iterrows():
    original_index = row.name
    limit_index = original_index - offset

    height = int(row['Message'].split('start_height: ')[1].split(',')[0])
    time_message_construct_request = row['Timestamp']


    if(height == 15):
        a = 0

    # find "SYNCPROFILING Sent block request for startheight" log in entire event_df, not just up to limit_index
    sent_block_request = event_df[event_df['Message'].str.contains(f'SYNCPROFILING Sent block request for startheight', na=False)]

    for j, row2 in sent_block_request.iterrows():

        # format: SYNCPROFILING Sent block request for startheight 1 to endheight 6 to peer
        start_height = int(row2['Message'].split('startheight ')[1].split(' to endheight')[0])
        end_height = int(row2['Message'].split('to endheight ')[1].split(' to peer')[0])

        if(row2.Timestamp < time_message_construct_request):
            continue

        if(row2.name > limit_index):
            a = 0
            #print("Error: Sent block request not found")
            #break

        if (height >= start_height and height < end_height):
            a = 0

            if(height == 15):
                a = 0

            time_message_sent_request = row2['Timestamp']

            x_bar_start = height - 0.5
            x_bar_end = height + 0.5
            y_bar_start = (time_message_construct_request - start_time).total_seconds()
            y_bar_end = (time_message_sent_request - start_time).total_seconds()
            a = 0

            # make a bar plot
            label = 'Time to construct request'
            if label not in used_labels:
                ax.bar((x_bar_end+x_bar_start)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_end-x_bar_start, label=label, color='tab:blue')
                used_labels[label] = True
            else:
                ax.bar((x_bar_end+x_bar_start)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_end-x_bar_start, color='tab:blue')

            break

        a = 0

    # find "SYNCPROFILING Sent block request for startheight" log in event_df up to limit_index
    #sent_block_request = event_df.iloc[:limit_index][event_df.iloc[:limit_index]['Message'].str.contains(f'SYNCPROFILING Sent block request for startheight', na=False)]




# find "Received block response for height" df
received_block_response = event_df[event_df['Message'].str.contains('Received block response for start height', na=False)]

for i, row in received_block_response.iterrows():
    original_index = row.name
    limit_index = original_index - offset

    start_height = int(row['Message'].split('Received block response for start height ')[1].split(' to end height')[0])
    end_height = int(row['Message'].split('to end height ')[1].split(' to peer')[0])
    
    # find "Sent block request for startheight ... to endheight ... to" in event_df
    sent_block_request = event_df[event_df['Message'].str.contains(f'Sent block request for startheight {start_height} to endheight {end_height} to', na=False)]

    if(len(sent_block_request) > 1):
        print("Error: More than one sent block request found")

    time_received_response = row['Timestamp']
    time_sent_request = sent_block_request.iloc[0]['Timestamp']

    x_bar_start = start_height - 0.5
    x_bar_end = end_height - 0.5
    y_bar_start = (time_sent_request - start_time).total_seconds()
    y_bar_end = (time_received_response - start_time).total_seconds()

    if(start_height == 5):
        a = 0

    # make a bar plot
    label = 'Time to receive response'
    if label not in used_labels:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, label=label, color='tab:orange')
        used_labels[label] = True
    else:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, color='tab:orange')


    a = 0

# find "SYNCPROFILING Deserialized blocks BlockRequest" df
deserialized_blocks = event_df[event_df['Message'].str.contains('SYNCPROFILING Deserialized blocks BlockRequest', na=False)]

for i, row in deserialized_blocks.iterrows():
    a = 0

    start_height = int(row['Message'].split('start_height: ')[1].split(',')[0])
    end_height = int(row['Message'].split('end_height: ')[1].split(' ')[0])

    # find "Received block response for height" in event_df
    received_block_response = event_df[event_df['Message'].str.contains(f'Received block response for start height {start_height} to end height {end_height}', na=False)]

    if(len(received_block_response) > 1):
        print("Error: More than one received block response found")

    time_received_response = received_block_response.iloc[0]['Timestamp']
    time_deserialized = row['Timestamp']

    x_bar_start = start_height - 0.5
    x_bar_end = end_height - 0.5
    y_bar_start = (time_received_response - start_time).total_seconds()
    y_bar_end = (time_deserialized - start_time).total_seconds()

    # make a bar plot
    label = 'Time to deserialize'
    if label not in used_labels:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, label=label, color='tab:green')
        used_labels[label] = True
    else:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, color='tab:green')


    a = 0

# find "SYNCPROFILING Check next block took" df
check_next_block = event_df[event_df['Message'].str.contains('SYNCPROFILING - starting check_next_block for block', na=False)]

for i, row in check_next_block.iterrows():
    # format: SYNCPROFILING Check next block took: 10911792ns for height 1
    height = int(row['Message'].split('block ')[2])

    # find SYNCPROFILING - ending check_next_block for block
    check_next_block_done = event_df[event_df['Message'] == f'SYNCPROFILING - ending check_next_block for block {height}']

    if(len(check_next_block_done) > 1):
        print("Error: More than one check next block done found")

    time_check_next_block_done = check_next_block_done.iloc[0]['Timestamp']
    time_check_next_block_start = row['Timestamp']

    x_bar_start = height - 0.5
    x_bar_end = height + 0.5
    y_bar_start = (time_check_next_block_start - start_time).total_seconds()
    y_bar_end = (time_check_next_block_done - start_time).total_seconds()

    # make a bar plot
    label = 'Check next block time'
    if label not in used_labels:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, label=label, color='tab:red')
        used_labels[label] = True
    else:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, color='tab:red')

    a = 0

prev_bottom = 0

# find "Advanced to block" df
advanced_to_block = event_df[event_df['Message'].str.contains('Advanced to block', na=False)]

for i, row in advanced_to_block.iterrows():
    # Advanced to block 1 at round 4 
    height = int(row['Message'].split('Advanced to block ')[1].split(' at round')[0])

    # find SYNCPROFILING - ending check_next_block for block
    check_next_block_done = event_df[event_df['Message'] == f'SYNCPROFILING - ending check_next_block for block {height}']

    if(len(check_next_block_done) > 1):
        print("Error: More than one start of ending check_next_block found")

    time_advanced_to_block_done = row['Timestamp']
    time_advanced_to_block_start = check_next_block_done.iloc[0]['Timestamp']

    x_bar_start = height - 0.5
    x_bar_end = height + 0.5
    y_bar_start = (time_advanced_to_block_start - start_time).total_seconds()
    y_bar_end = (time_advanced_to_block_done - start_time).total_seconds()

    # make a bar plot
    label = 'Advanced to block'
    if label not in used_labels:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, label=label, color='tab:brown')
        used_labels[label] = True
    else:
        ax.bar((x_bar_start+x_bar_end)/2, y_bar_end-y_bar_start, bottom=y_bar_start, width=x_bar_start-x_bar_end, color='tab:brown')

    a = 0

# plot legend
ax.legend()
# Set the title and labels
ax.set_title(f"Syncing of validator {val_index}")
ax.set_xlabel("Block height index")
ax.set_ylabel("Time (seconds)")

# show the plot
plt.show()

a = 0
# Save the figure
fig.savefig(f'1_stacked_bar_chart_block_times_val{val_index}.png')

