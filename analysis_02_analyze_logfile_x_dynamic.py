import os
import pandas as pd
import matplotlib.pyplot as plt
import copy
import numpy as np

# Set the variables
num_val = 15
val_index = 0
log_file_name = f"prepared_logs_{val_index}.log"
log_file_path = os.path.join(os.getcwd(), "aws-logs", log_file_name)

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

class BlockRangeRequest:
    def __init__(self, time_starting_to_find_sync_peers):
        self.times_starting_to_find_sync_peers = [time_starting_to_find_sync_peers]
        self.time_sync_peers_found = None
        self.times_to_construct_requests = []
        self.time_to_construct_combined_request = None
        self.time_sent_request = None
        self.block_start_height = None
        self.block_end_height = None
        self.time_received_response = None
        self.time_deserialized = None
        self.time_check_next_block_done = []
        self.times_message_advanced_to_block = []
        self.times_extra_message_advanced_to_block = []

    def get_time_to_find_sync_peers(self):
        # todo discuss with victor about multiple start times
        return self.time_sync_peers_found - self.times_starting_to_find_sync_peers[0]

    def get_times_to_construct_requests(self):
        times = []
        last_time = self.time_sync_peers_found
        for time in self.times_to_construct_requests:
            times.append(time - last_time)
            last_time = time
        return times
    
    def get_time_to_construct_combined_request(self):
        return self.time_to_construct_combined_request - self.times_to_construct_requests[-1]
                
    def get_time_to_sent_request(self):
        return self.time_sent_request - self.time_to_construct_combined_request
    
    def get_time_to_received_response(self):
        return self.time_received_response - self.time_sent_request
    
    def get_time_to_deserialized(self):
        return self.time_deserialized - self.time_received_response
    
    def get_times_check_next_block_done(self):
        times = []
        last_time = self.time_deserialized
        for i, time in enumerate(self.time_check_next_block_done):
            times.append(time - last_time)
            last_time = self.times_extra_message_advanced_to_block[i]
        return times
    
    def get_times_extra_message_advanced_to_block(self):
        times = []
        last_time = self.time_check_next_block_done[0]
        for i, time in enumerate(self.times_extra_message_advanced_to_block):
            times.append(time - last_time)
            last_time = self.time_check_next_block_done[i]
        return times
    
    def get_total_time(self):
        return self.times_extra_message_advanced_to_block[-1] - self.times_starting_to_find_sync_peers[0]
    
# Initialize dictionaries to store times
advanced_block_times = []
blockRangeRequests = []

current_blockRangeRequest = None

for index, row in event_df.iterrows():
    message = row['Message']
    timestamp = row['Timestamp']

    # check if message is profiling - starting proposal generation for round {round_number}
    if "SYNCPROFILING Starting to find sync peers..." in message:
        if(current_blockRangeRequest is None):
            current_blockRangeRequest = BlockRangeRequest(timestamp)
        else:
            if(len(current_blockRangeRequest.times_message_advanced_to_block) == 0):
                current_blockRangeRequest.times_starting_to_find_sync_peers.append(timestamp)
            else:
                blockRangeRequests.append(current_blockRangeRequest)
                current_blockRangeRequest = BlockRangeRequest(timestamp)

        continue

    if "SYNCPROFILING Time to find sync peers" in message:
        current_blockRangeRequest.time_sync_peers_found = timestamp
        continue

    if "SYNCPROFILING Time to construct request:" in message:
        current_blockRangeRequest.times_to_construct_requests.append(timestamp)
        continue

    if "SYNCPROFILING Time to construct requests:" in message:
        current_blockRangeRequest.time_to_construct_combined_request = timestamp
        continue

    if "SYNCPROFILING Sent block request for startheight" in message:
        start_height = int(message.split('startheight ')[1].split(' to')[0])
        end_height = int(message.split('to endheight ')[1].split(' to')[0])
        current_blockRangeRequest.block_start_height = start_height
        current_blockRangeRequest.block_end_height = end_height
        current_blockRangeRequest.time_sent_request = timestamp
        continue

    if "SYNCPROFILING Received block response" in message:
        current_blockRangeRequest.time_received_response = timestamp
        continue

    if "SYNCPROFILING Deserialized blocks" in message:
        current_blockRangeRequest.time_deserialized = timestamp
        continue

    # not yet implemented: SYNCPROFILING NUM BLOCKS PENDING IN RESPONSES
    # not yet implemented: SYNCPROFILING IS THE NEXT BLOCK IN THE CURRENT RESPONSES? false, next next true

    if "SYNCPROFILING CHECK NEXT BLOCK TIME" in message:
        current_blockRangeRequest.time_check_next_block_done.append(timestamp)
        continue

    if "Advanced to block " in message:
        current_blockRangeRequest.times_message_advanced_to_block.append(timestamp)
        continue

    if "SYNCPROFILING ADVANCE TO NEXT BLOCK TIME" in message:
        current_blockRangeRequest.times_extra_message_advanced_to_block.append(timestamp)

a = 0

blockRangeRequests

blockRangeRequests_durations = []
for blockRangeRequest in blockRangeRequests:
    blockRangeRequests_durations.append(blockRangeRequest.get_total_time().total_seconds())

# Extract all times\
time_to_find_sync_peers = []
times_to_construct_requests = []
time_to_construct_combined_request = []
time_to_sent_request = []
time_to_received_response = []
time_to_deserialized = []
times_check_next_block_done = []
times_extra_message_advanced_to_block = []
unaccounted_times = []

#previous_block_end_time = 0 
for blockRangeRequest in blockRangeRequests:
    this_time_to_find_sync_peers = blockRangeRequest.get_time_to_find_sync_peers()
    this_times_to_construct_requests = blockRangeRequest.get_times_to_construct_requests()
    this_time_to_construct_combined_request = blockRangeRequest.get_time_to_construct_combined_request()
    this_time_to_sent_request = blockRangeRequest.get_time_to_sent_request()
    this_time_to_received_response = blockRangeRequest.get_time_to_received_response()
    this_time_to_deserialized = blockRangeRequest.get_time_to_deserialized()
    this_times_check_next_block_done = blockRangeRequest.get_times_check_next_block_done()
    this_times_extra_message_advanced_to_block = blockRangeRequest.get_times_extra_message_advanced_to_block()
    this_unaccounted_time = blockRangeRequest.get_total_time().total_seconds() - this_time_to_find_sync_peers.total_seconds() - sum([time.total_seconds() for time in this_times_to_construct_requests]) - this_time_to_construct_combined_request.total_seconds() - this_time_to_sent_request.total_seconds() - this_time_to_received_response.total_seconds() - this_time_to_deserialized.total_seconds() - sum([time.total_seconds() for time in this_times_check_next_block_done]) - sum([time.total_seconds() for time in this_times_extra_message_advanced_to_block])

    time_to_find_sync_peers.append(this_time_to_find_sync_peers.total_seconds())
    times_to_construct_requests.append([time.total_seconds() for time in this_times_to_construct_requests])
    time_to_construct_combined_request.append(this_time_to_construct_combined_request.total_seconds())
    time_to_sent_request.append(this_time_to_sent_request.total_seconds())
    time_to_received_response.append(this_time_to_received_response.total_seconds())
    time_to_deserialized.append(this_time_to_deserialized.total_seconds())
    times_check_next_block_done.append([time.total_seconds() for time in this_times_check_next_block_done])
    times_extra_message_advanced_to_block.append([time.total_seconds() for time in this_times_extra_message_advanced_to_block])
    unaccounted_times.append(this_unaccounted_time)

def flatten_list_of_lists(lol):
    return [item for sublist in lol for item in sublist]

# Plotting
fig, ax = plt.subplots(figsize=(10, 7))

# Initialize the bottom stack array for 114 blocks
bottoms = np.zeros(len(blockRangeRequests))

# Define colors for each category
sync_peers_color = 'tab:blue'
construct_requests_color = 'tab:orange'
construct_combined_color = 'tab:green'
sent_request_color = 'tab:red'
received_response_color = 'tab:purple'
deserialized_color = 'tab:brown'
check_next_color = 'tab:pink'
extra_message_advanced_color = 'tab:gray'
unaccounted_color = 'tab:olive'

# todo, now change the code below - basically, print depending on if it is one value or a list of values

# Iterate over each blockRangeRequest
for i in range(len(blockRangeRequests)):

    # Set the x position and width of each box based on the block start and end heights
    block_start = blockRangeRequests[i].block_start_height
    block_end = blockRangeRequests[i].block_end_height
    block_width = block_end - block_start

    # Plot time to find sync peers (single value)
    ax.bar(block_start+block_width/2, time_to_find_sync_peers[i], bottom=bottoms[i], width=block_width, 
           label='Time to find sync peers' if i == 0 else "", color=sync_peers_color)
    bottoms[i] += time_to_find_sync_peers[i]
    
    # Plot times to construct requests (list of values)
    for construct_time in times_to_construct_requests[i]:
        ax.bar(block_start+block_width/2, construct_time, bottom=bottoms[i], width=block_width,
               label='Time to construct requests' if i == 0 and construct_time == times_to_construct_requests[i][0] else "", color=construct_requests_color)
        bottoms[i] += construct_time

    # Plot time to construct combined request (single value)
    ax.bar(block_start+block_width/2, time_to_construct_combined_request[i], bottom=bottoms[i], width=block_width,
           label='Time to construct combined request' if i == 0 else "", color=construct_combined_color)
    bottoms[i] += time_to_construct_combined_request[i]

    # Plot time to send request (single value)
    ax.bar(block_start+block_width/2, time_to_sent_request[i], bottom=bottoms[i], width=block_width,
           label='Time to send request' if i == 0 else "", color=sent_request_color)
    bottoms[i] += time_to_sent_request[i]

    # Plot time to receive response (single value)
    ax.bar(block_start+block_width/2, time_to_received_response[i], bottom=bottoms[i], width=block_width,
           label='Time to receive response' if i == 0 else "", color=received_response_color)
    bottoms[i] += time_to_received_response[i]

    # Plot time to deserialize response (single value)
    ax.bar(block_start+block_width/2, time_to_deserialized[i], bottom=bottoms[i], width=block_width,
           label='Time to deserialize' if i == 0 else "", color=deserialized_color)
    bottoms[i] += time_to_deserialized[i]

    # Alternate between check next block and extra message advanced to block (lists of values)
    for check_time, extra_time in zip(times_check_next_block_done[i], times_extra_message_advanced_to_block[i]):
        ax.bar(block_start+block_width/2, check_time, bottom=bottoms[i], width=block_width,
               label='Check next block time' if i == 0 and check_time == times_check_next_block_done[i][0] else "", color=check_next_color)
        bottoms[i] += check_time

        ax.bar(block_start+block_width/2, extra_time, bottom=bottoms[i], width=block_width,
               label='Advanced to block' if i == 0 and extra_time == times_extra_message_advanced_to_block[i][0] else "", color=extra_message_advanced_color)
        bottoms[i] += extra_time

    # Plot unaccounted time (single value)
    ax.bar(block_start+block_width/2, unaccounted_times[i], bottom=bottoms[i], width=block_width,
           label='Unaccounted time' if i == 0 else "", color=unaccounted_color)
    bottoms[i] += unaccounted_times[i]

# Add labels and title
ax.set_xlabel('Block Height')
ax.set_ylabel('Time (s)')
ax.set_title(f'Stacked Bar Chart of Block Times, client {val_index}')
ax.legend()

# Show the plot
plt.show()

# Save the figure
fig.savefig(f'1_stacked_bar_chart_block_times_val{val_index}.png')

for i in range(len(blockRangeRequests)):
    # print start and end height
    print(f"Block Range Request {i}:")
    print(f"Start Height: {blockRangeRequests[i].block_start_height}")
    print(f"End Height: {blockRangeRequests[i].block_end_height}")
    