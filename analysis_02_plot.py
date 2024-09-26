import os
import pandas as pd
import matplotlib.pyplot as plt
import copy
import numpy as np

# Set the variables
num_val = 15
val_index = 3
log_file_name = f"prepared_logs_{val_index}.log"
log_file_path = os.path.join(os.getcwd(), "aws-logs5", log_file_name)

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
                ax.bar(height, y_bar_end-y_bar_start, bottom=y_bar_start, width=1, label=label, color='tab:blue')
                used_labels[label] = True
            else:
                ax.bar(height, y_bar_end-y_bar_start, bottom=y_bar_start, width=1, color='tab:blue')

            break

        a = 0

    # find "SYNCPROFILING Sent block request for startheight" log in event_df up to limit_index
    #sent_block_request = event_df.iloc[:limit_index][event_df.iloc[:limit_index]['Message'].str.contains(f'SYNCPROFILING Sent block request for startheight', na=False)]



    a = 0

a = 0



prev_bottom = 0

for j, tryBlockSyncCall in enumerate(tryBlockSyncCalls):
    if(not tryBlockSyncCall.prepared_zero_block_requests):
        #get_time_to_find_sync_peers, get_times_to_construct_requests, get_unaccounted_time_in_prepare_block_requests
        start_height = tryBlockSyncCall.block_requests_start_height
        end_height = tryBlockSyncCall.block_requests_end_height
        if(start_height == 35):
            a = 0
        time_to_find_sync_peers = tryBlockSyncCall.get_time_to_find_sync_peers().total_seconds()
        times_to_construct_requests = tryBlockSyncCall.get_times_to_construct_requests()
        unaccounted_time = tryBlockSyncCall.get_unaccounted_time_in_prepare_block_requests_seconds()
        combined_time = tryBlockSyncCall.get_combined_time_to_construct_requests().total_seconds()
        # assuming combined_time is small enought hat we can just proceed with it
        
        # plot a bar stack. x from start_height to end_height, y from 0 to time_to_find_sync_peers, combined_time. color should be blue
        width = end_height-start_height
        label = 'Time to find sync peers'
        if label not in used_labels:
            ax.bar(start_height + width / 2, combined_time, bottom=prev_bottom, width=width, label=label, color='tab:blue')
            used_labels[label] = True
        else:
            ax.bar(start_height + width / 2, combined_time, bottom=prev_bottom, width=width, color='tab:blue')

        # get times to send requests
        times_send, ranges = tryBlockSyncCall.get_times_to_send_requests()
        times_send_seconds = [time.total_seconds() for time in times_send]
        #times_to_send_requests, ranges = tryBlockSyncCall.get_times_to_send_requests()

        bottoms = np.zeros(len(times_send_seconds))

        times_send_seconds_bar_bottoms = []
        prev_bottom += combined_time
        for i, time in enumerate(times_send_seconds):
            start_height = ranges[i][0]
            end_height = ranges[i][1]
            width = end_height-start_height

            label = 'Time to send request'
            if label not in used_labels:
                ax.bar(start_height + width / 2, time, bottom=prev_bottom, width=width, label=label, color='tab:orange')
                used_labels[label] = True
            else:
                ax.bar(start_height + width / 2, time, bottom=prev_bottom, width=width, color='tab:orange')


            prev_bottom += time
            times_send_seconds_bar_bottoms.append(prev_bottom)
            bottoms[i] = prev_bottom

        # get times to receive responses
        times_receive = []
        for blockRangeRequest in tryBlockSyncCall.BlockRangeRequests.values():
            times_receive.append(blockRangeRequest.get_time_to_received_response())
        times_receive_seconds = [time.total_seconds() for time in times_receive]

        for i, time in enumerate(times_receive_seconds):
            start_height = ranges[i][0]
            end_height = ranges[i][1]
            width = end_height-start_height


            if(start_height == 50 or start_height == 51):
                a = 0

            label = 'Time to receive response'
            if label not in used_labels:
                ax.bar(start_height + width / 2, time, bottom=times_send_seconds_bar_bottoms[i], width=width, label=label, color='tab:green')
                used_labels[label] = True
            else:
                ax.bar(start_height + width / 2, time, bottom=times_send_seconds_bar_bottoms[i], width=width, color='tab:green')


            bottoms[i] += time

        times_deserialized = []
        for blockRangeRequest in tryBlockSyncCall.BlockRangeRequests.values():
            times_deserialized.append(blockRangeRequest.get_time_to_deserialized())
        times_deserialized_seconds = [time.total_seconds() for time in times_deserialized]

        for i, time in enumerate(times_deserialized_seconds):
            start_height = ranges[i][0]
            end_height = ranges[i][1]
            width = end_height-start_height

            label = 'Time to deserialize'
            if label not in used_labels:
                ax.bar(start_height+width/2, time, bottom=bottoms[i], width=width, label=label, color='tab:red')
                used_labels[label] = True
            else:
                ax.bar(start_height+width/2, time, bottom=bottoms[i], width=width, color='tab:red')

            bottoms[i] += time





        # after all deserialization is done, stair wise check next block and advance to block
        times_check_next_block_done = tryBlockSyncCall.get_times_check_next_block_done()
        times_check_next_block_done_seconds = [time.total_seconds() for time in times_check_next_block_done]

        times_extra_message_advanced_to_block = tryBlockSyncCall.get_times_advanced_to_block()
        times_extra_message_advanced_to_block_seconds = [time.total_seconds() for time in times_extra_message_advanced_to_block]

        start_time_sequential_tasks = max(bottoms)

        for i, time in enumerate(times_check_next_block_done_seconds):
            start_height = tryBlockSyncCall.block_requests_start_height + i
            end_height = start_height + 1
            width = end_height-start_height

            label = 'Check next block time'
            if label not in used_labels:
                ax.bar(start_height+width/2, time, bottom=start_time_sequential_tasks, width=width, label=label, color='tab:purple')
                used_labels[label] = True
            else:
                ax.bar(start_height+width/2, time, bottom=start_time_sequential_tasks, width=width, color='tab:purple')

            start_time_sequential_tasks += time
            time = times_extra_message_advanced_to_block_seconds[i]

            label = 'Advanced to block'
            if label not in used_labels:
                ax.bar(start_height+width/2, time, bottom=start_time_sequential_tasks, width=width, label=label, color='tab:brown')
                used_labels[label] = True
            else:
                ax.bar(start_height+width/2, time, bottom=start_time_sequential_tasks, width=width, color='tab:brown')

            start_time_sequential_tasks += time
        
        prev_bottom = start_time_sequential_tasks

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

