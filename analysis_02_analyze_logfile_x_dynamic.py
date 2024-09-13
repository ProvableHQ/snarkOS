import os
import pandas as pd
import matplotlib.pyplot as plt
import copy
import numpy as np

# Set the variables
num_val = 15
val_index = 3
log_file_name = f"prepared_logs_{val_index}.log"
log_file_path = os.path.join(os.getcwd(), "aws-logs10", log_file_name)

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

class TryBlockSyncCall:
    def __init__(self, startTime):
        self.startTime = startTime
        self.time_start_find_sync_peers = None
        self.time_sync_peers_found = None
        self.time_end_prepare_block_requests = None
        self.prepared_zero_block_requests = None
        self.BlockRequests = {}
        self.time_log_constructed_block_requests = None
        self.block_requests_raw_aggegarate_time = None
        self.block_requests_start_height = None
        self.block_requests_end_height = None
        self.BlockRangeRequests = {}
        self.time_start_sending_block_requests = None

    def get_time_to_find_sync_peers(self):
        return self.time_sync_peers_found - self.time_start_find_sync_peers
    
    def get_times_to_construct_requests(self):
        times = []
        # iterate over all block requests
        for blockRequest in self.BlockRequests.values():
            times.append(blockRequest.get_time_to_construct_raw()*10**-9)
        return times
    
    def get_combined_time_to_construct_requests(self):
        return self.time_log_constructed_block_requests - self.startTime

    def _get_time_to_end_prepare_block_requests(self):
        return self.time_end_prepare_block_requests - self.startTime
    
    def get_unaccounted_time_in_prepare_block_requests_seconds(self):
        return self._get_time_to_end_prepare_block_requests().total_seconds() - self.get_time_to_find_sync_peers().total_seconds() - sum(self.get_times_to_construct_requests())

    def get_times_to_send_requests(self):
        times = []
        ranges = []
        start_send_time = self.time_start_sending_block_requests
        for blockRangeRequest in self.BlockRangeRequests.values():
            this_time = blockRangeRequest.get_time_in_sent_request(start_send_time)
            times.append(this_time)
            start_height = blockRangeRequest.start_height
            end_height = blockRangeRequest.end_height
            ranges.append((start_height, end_height))
            start_send_time = blockRangeRequest.time_sent_request
        return times, ranges
    
    def get_times_check_next_block_done_old(self): # can be deleted
        times = []
        previous_time = None
        # find time when the last block range request is done deserilizing
        for blockRangeRequest in self.BlockRangeRequests.values():
            if(previous_time is None):
                previous_time = blockRangeRequest.time_deserialized
            elif(blockRangeRequest.time_deserialized > previous_time):
                previous_time = blockRangeRequest.time_deserialized
                
        for blockRequest in self.BlockRequests.values():
            # corresponding_block_check_next_block_time
            times.append(blockRequest.corresponding_block_check_next_block_time - previous_time)
            previous_time = blockRequest.corresponding_block_check_next_block_time

        for blockRangeRequest in self.BlockRangeRequests.values():
            times.append(blockRangeRequest.get_time_to_check_next_block_done())
        return times
    
    def get_times_check_next_block_done(self):
        times = []
        for blockRequest in self.BlockRequests.values():
            times.append(blockRequest.get_time_to_check_next_block_done())
        return times
    
    def get_times_advanced_to_block(self):
        times = []
        for blockRequest in self.BlockRequests.values():
            times.append(blockRequest.get_time_to_advanced_to_block())
        return times



class BlockRequest:
    def __init__(self, time_constructed, height, raw_time_to_construct):
        self.time_constructed = time_constructed
        self.height = height
        self.raw_time_to_construct = raw_time_to_construct

        self.time_processing_block_response = None
        self.corresponding_block_check_next_block_time = None
        self.corresponding_block_advanced_to_block_time = None
        self.corresponding_block_advanced_to_block_time2 = None

    def get_time_to_construct_raw(self):
        return self.raw_time_to_construct
    
    def get_time_to_sent_request(self, previous_log_time):
        return self.time_sent_request - previous_log_time
    
    def get_time_to_received_response(self):
        return self.time_received_response - self.time_sent_request
    
    def get_time_to_check_next_block_done(self):
        # assuming sequential, not parallel
        return self.corresponding_block_check_next_block_time - self.time_processing_block_response
    
    def get_time_to_advanced_to_block(self):
        # assuming sequential, not parallel
        if self.corresponding_block_advanced_to_block_time is None:
            return pd.Timedelta(seconds=0)
        if self.corresponding_block_check_next_block_time is None:
            return pd.Timedelta(seconds=0)
        return self.corresponding_block_advanced_to_block_time - self.corresponding_block_check_next_block_time

    def get_times_to_received_response_seconds(self):
        times = []
        for blockRequest in self.BlockRequests:
            times.append(blockRequest.get_time_to_received_response())
        return times
    
    def get_times_to_deserialized(self):
        times = []
        for blockRangeRequest in self.BlockRangeRequests:
            times.append(blockRangeRequest.get_time_to_deserialized())
        return times
    
    def get_time_to_check_next_block_done(self):
        if self.corresponding_block_check_next_block_time is None:
            # return timedelta with 0 seconds
            return pd.Timedelta(seconds=0)
        if self.time_processing_block_response is None:
            return pd.Timedelta(seconds=0)
        return self.corresponding_block_check_next_block_time - self.time_processing_block_response
    
    def get_time_to_advanced_to_block_done(self):
        return self.corresponding_block_advanced_to_block_time - self.corresponding_block_check_next_block_time

class BlockRangeRequest:
    def __init__(self, time_sent_request, start_height, end_height, block_requests):
        self.time_sent_request = time_sent_request
        self.start_height = start_height
        self.end_height = end_height
        self.block_requests = block_requests
        self.time_received_response = None
        self.time_deserialized = None

    def get_time_in_construct_raw_requests(self):
        return sum([block_request.raw_time_to_construct for block_request in self.block_requests])

    def get_time_in_sent_request(self, previous_time):
        return self.time_sent_request - previous_time
    
    def get_time_to_received_last_response(self):
        # find the last received response for the block requests
        last_received_response = None
        index_last_received_response = None
        for i, block_request in enumerate(self.block_requests):
            if(block_request.time_received_response is not None):
                last_received_response = block_request.time_received_response
                index_last_received_response = i
            elif(block_request.time_received_response > last_received_response):
                last_received_response = block_request.time_received_response
                index_last_received_response = i
        
        time_send = self.block_requests[index_last_received_response].time_sent_request
        return last_received_response - time_send
    
    def get_time_to_deserialized(self): # assuming deserialization is triggered after receiving a response
        return self.time_deserialized - self.time_received_response
    
    def get_time_to_check_next_block_done(self):
        time = 0
        for i, block_request in enumerate(self.block_requests):
            time += block_request.get_time_to_check_next_block_done()
        return time
    
    def get_time_to_advanced_to_block(self):
        time = 0
        for i, block_request in enumerate(self.block_requests):
            time += block_request.get_time_to_advanced_to_block()
        return time
    
    def get_time_to_received_response(self):
        return self.time_received_response - self.time_sent_request
    
    def get_time_to_deserialized(self):
        return self.time_deserialized - self.time_received_response
    
    def get_time_to_find_sync_peers(self):
        # todo discuss with victor about multiple start times
        return self.time_sync_peers_found - self.times_starting_to_find_sync_peers[0]  
    
    def get_time_to_received_response(self):
        return self.time_received_response - self.time_sent_request
    
    def get_time_to_deserialized(self):
        return self.time_deserialized - self.time_received_response
    
# Initialize dictionaries to store times
advanced_block_times = []
blockRangeRequests = []

tryBlockSyncCalls = []
current_tryBlockSyncCall = None

def get_blockRequest_by_height(currentTryBlockSyncCall, height):
    if currentTryBlockSyncCall is not None:
        if height in currentTryBlockSyncCall.BlockRequests:
            return currentTryBlockSyncCall.BlockRequests[height]
    for tryBlockSyncCall in tryBlockSyncCalls:
        for blockRequest in tryBlockSyncCall.BlockRequests.values():
            if(blockRequest.height == height):
                return blockRequest
    return None

for index, row in event_df.iterrows():
    message = row['Message']
    timestamp = row['Timestamp']

    print(message)

    if "SYNCPROFILING try_block_sync" in message:
        # todo check if add to list
        if(current_tryBlockSyncCall is not None):
            tryBlockSyncCalls.append(current_tryBlockSyncCall)
            current_tryBlockSyncCall = None
        current_tryBlockSyncCall = TryBlockSyncCall(timestamp)
        continue

    if "SYNCPROFILING Starting to find sync peers..." in message:
        current_tryBlockSyncCall.time_start_find_sync_peers = timestamp
        continue

    if "SYNCPROFILING Time to find sync peers" in message:
        current_tryBlockSyncCall.time_sync_peers_found = timestamp
        continue

    if "SYNCPROFILING End of prepare_block_requests" in message:
        current_tryBlockSyncCall.time_end_prepare_block_requests = timestamp
        # get number of block requests
        block_requests_len = int(message.split('prepared ')[1].split(' block requests')[0])
        if(block_requests_len == 0):
            current_tryBlockSyncCall.prepared_zero_block_requests = True
        continue

    if "SYNCPROFILING Time to construct request:" in message:
        start_height = int(message.split('start_height: ')[1].split(', end_height')[0])
        # end_height = int(message.split('end_height: ')[1].split(' ')[0]) # assumed to be 1 higher
        raw_time_to_construct_request = int(message.split('Time to construct request: ')[1].split('ns')[0])

        br = BlockRequest(timestamp, start_height, raw_time_to_construct_request)
        current_tryBlockSyncCall.BlockRequests[start_height] = br
        current_tryBlockSyncCall.prepared_zero_block_requests = False
        continue

    if "SYNCPROFILING Time to construct requests:" in message:
        current_tryBlockSyncCall.time_log_constructed_block_requests = timestamp
        current_tryBlockSyncCall.raw_aggegarate_time_to_construct_block_requests = int(message.split('Time to construct requests: ')[1].split('ns')[0])
        current_tryBlockSyncCall.block_requests_start_height = int(message.split('start_height: ')[1].split(', end_height')[0])
        current_tryBlockSyncCall.block_requests_end_height = int(message.split('end_height: ')[1].split(' ')[0])
                                    
        continue

    if "SYNCPROFILING Sending block requests" in message:
        current_tryBlockSyncCall.time_start_sending_block_requests = timestamp
        continue

    if "SYNCPROFILING Sent block request for startheight" in message:
        start_height = int(message.split('startheight ')[1].split(' to')[0])
        end_height = int(message.split('to endheight ')[1].split(' to')[0])
        # extract block requests
        block_requests = []
        for i in range(start_height, end_height):
            current_tryBlockSyncCall.BlockRequests[i].time_sent_request = timestamp
            block_requests.append(current_tryBlockSyncCall.BlockRequests[i])
        brr = BlockRangeRequest(timestamp, start_height, end_height, block_requests)
        current_tryBlockSyncCall.BlockRangeRequests[start_height] = brr

        continue

    if "SYNCPROFILING Received block response for height" in message:
        #current_blockRangeRequest.time_received_response = timestamp
        height = int(message.split('height ')[1].split(' ')[0])
        current_tryBlockSyncCall.BlockRangeRequests[height].time_received_response = timestamp
        continue

    if "SYNCPROFILING Deserialized blocks BlockRequest { start_height" in message:
        start_height = int(message.split('start_height: ')[1].split(', end_height')[0])
        current_tryBlockSyncCall.BlockRangeRequests[start_height].time_deserialized = timestamp
        continue

    if "SYNCPROFILING NUM BLOCKS PENDING IN RESPONSES" in message:
        continue
    if "SYNCPROFILING IS THE NEXT BLOCK IN THE CURRENT RESPONSES?" in message:
        continue

    if "SYNCPROFILING Processing block response for height" in message:
        height = int(message.split('height ')[1].split(' ')[0])
        get_blockRequest_by_height(current_tryBlockSyncCall, height).time_processing_block_response = timestamp
        a = 0

    if "SYNCPROFILING CHECK NEXT BLOCK TIME" in message:
        height = int(message.split('height ')[1])
        br = get_blockRequest_by_height(current_tryBlockSyncCall, height)
        if(br.height == 53):
            a = 0
        get_blockRequest_by_height(current_tryBlockSyncCall, height).corresponding_block_check_next_block_time = timestamp
        continue

    if "Advanced to block" in message:
        advanced_block_times.append(timestamp)
        block_height = int(message.split('Advanced to block ')[1].split(' ')[0])
        get_blockRequest_by_height(current_tryBlockSyncCall, height).corresponding_block_advanced_to_block_time = timestamp
        continue

    if "SYNCPROFILING ADVANCE TO NEXT BLOCK TIME" in message:
        block_height = int(message.split('height ')[1])
        get_blockRequest_by_height(current_tryBlockSyncCall, height).corresponding_block_advanced_to_block_time2 = timestamp
        continue


blockRangeRequests_durations = []
for tryBlockSyncCall in tryBlockSyncCalls:
    pass
    #blockRangeRequests_durations.append(tryBlockSyncCall.get_total_time().total_seconds())

# start a bar chart figure
fig, ax = plt.subplots(figsize=(10, 7))
# legend
ax.legend()
used_labels = {}

prev_bottom = 0

for j, tryBlockSyncCall in enumerate(tryBlockSyncCalls):
    if(not tryBlockSyncCall.prepared_zero_block_requests):
        if(j >= 1):
            a = 0
        #get_time_to_find_sync_peers, get_times_to_construct_requests, get_unaccounted_time_in_prepare_block_requests
        start_height = tryBlockSyncCall.block_requests_start_height
        end_height = tryBlockSyncCall.block_requests_end_height
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
ax.set_title(f"Syncing of client {val_index}")
ax.set_xlabel("Block height index")
ax.set_ylabel("Time (seconds)")

# show the plot
plt.show()

a = 0
# Save the figure
fig.savefig(f'1_stacked_bar_chart_block_times_val{val_index}.png')

