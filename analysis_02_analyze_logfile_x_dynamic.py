import os
import pandas as pd
import matplotlib.pyplot as plt
import copy
import numpy as np

# Set the variables
num_val = 15
val_index = 0
log_file_name = f"prepared_logs_{val_index}.log"
log_file_path = os.path.join(os.getcwd(), "aws-logs7", log_file_name)

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
    def __init__(self, time_constructed, start_height, end_height, raw_time_to_construct):
        self.time_constructed = time_constructed
        self.start_height = start_height
        self.end_height = end_height
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
        return self.corresponding_block_advanced_to_block_time - self.corresponding_block_check_next_block_time

    def get_times_in_sent_request_seconds(self):
        times = []
        previous_log_time = self.time_start_sending_block_requests
        for blockRequest in self.BlockRequests:
            times.append(blockRequest.get_time_to_sent_request(previous_log_time))
            previous_log_time = blockRequest.time_sent_request
        return times
    
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
    

    

        

    
    # old functions, can likely be removed
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

tryBlockSyncCalls = []
current_tryBlockSyncCall = None

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
        end_height = int(message.split('end_height: ')[1].split(' ')[0])
        raw_time_to_construct_request = int(message.split('Time to construct request: ')[1].split('ns')[0])

        br = BlockRequest(timestamp, start_height, end_height, raw_time_to_construct_request)
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
        start_key = list(current_tryBlockSyncCall.BlockRangeRequests.keys())[0]
        for key in current_tryBlockSyncCall.BlockRangeRequests.keys():
            if(key <= height):
                start_key = key
            else:
                break
        current_tryBlockSyncCall.BlockRangeRequests[start_key].block_requests[height-start_key].time_processing_block_response = timestamp
        a = 0

    if "SYNCPROFILING CHECK NEXT BLOCK TIME" in message:
        height = int(message.split('height ')[1])
        # find corresponding start block index in current_tryBlockSyncCall.BlockRangeRequests.keys()
        start_key = list(current_tryBlockSyncCall.BlockRangeRequests.keys())[0]
        for key in current_tryBlockSyncCall.BlockRangeRequests.keys():
            if(key <= height):
                start_key = key
            else:
                break
        current_tryBlockSyncCall.BlockRangeRequests[start_key].block_requests[height-start_key].corresponding_block_check_next_block_time = timestamp
        continue

    if "Advanced to block" in message:
        advanced_block_times.append(timestamp)
        block_height = int(message.split('Advanced to block ')[1].split(' ')[0])
        start_key = list(current_tryBlockSyncCall.BlockRangeRequests.keys())[0]
        for key in current_tryBlockSyncCall.BlockRangeRequests.keys():
            if(key <= height):
                start_key = key
            else:
                break
        current_tryBlockSyncCall.BlockRangeRequests[start_key].block_requests[height-start_key].corresponding_block_advanced_to_block_time = timestamp
        continue

    if "SYNCPROFILING ADVANCE TO NEXT BLOCK TIME" in message:
        block_height = int(message.split('height ')[1])
        start_key = list(current_tryBlockSyncCall.BlockRangeRequests.keys())[0]
        for key in current_tryBlockSyncCall.BlockRangeRequests.keys():
            if(key <= height):
                start_key = key
            else:
                break
        current_tryBlockSyncCall.BlockRangeRequests[start_key].block_requests[height-start_key].corresponding_block_advanced_to_block_time2 = timestamp
        continue

    a = 0


    # Rust code: info!("SYNCPROFILING End of prepare_block_requests, prepared {} block requests", block_requests.len());
    if "SYNCPROFILING End of prepare_block_requests" in message:
        current_blockRangeRequest.time_end_prepare_block_requests = timestamp
        # extract length of block requests from message
        block_requests_len = int(message.split('prepared ')[1].split(' block requests')[0])
        current_blockRangeRequest.block_requests_len = block_requests_len
        continue


    # check if message is profiling - starting proposal generation for round {round_number}
    if "SYNCPROFILING Starting to find sync peers..." in message:
        current_blockRangeRequest.times_starting_to_find_sync_peers.append(timestamp)
        #if(current_blockRangeRequest is None):
        #    current_blockRangeRequest = BlockRangeRequest(timestamp) # todo change logic below
        #else:
        #    if(len(current_blockRangeRequest.times_message_advanced_to_block) == 0):
        #        current_blockRangeRequest.times_starting_to_find_sync_peers.append(timestamp)
        #    else:
        #        blockRangeRequests.append(current_blockRangeRequest)
        #        current_blockRangeRequest = BlockRangeRequest(timestamp)

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

blockRangeRequests_durations = []
for tryBlockSyncCall in tryBlockSyncCalls:
    pass
    #blockRangeRequests_durations.append(tryBlockSyncCall.get_total_time().total_seconds())

# start a bar chart figure
fig, ax = plt.subplots(figsize=(10, 7))
# legend
ax.legend()

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
        ax.bar(start_height+width/2, combined_time, bottom=prev_bottom, width=width, label='Time to find sync peers', color='tab:blue')

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
            ax.bar(start_height+width/2, time, bottom=prev_bottom, width=width, label='Time to send request', color='tab:orange')
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
            ax.bar(start_height+width/2, time, bottom=times_send_seconds_bar_bottoms[i], width=width, label='Time to receive response', color='tab:green')
            bottoms[i] += time

        times_deserialized = []
        for blockRangeRequest in tryBlockSyncCall.BlockRangeRequests.values():
            times_deserialized.append(blockRangeRequest.get_time_to_deserialized())
        times_deserialized_seconds = [time.total_seconds() for time in times_deserialized]

        for i, time in enumerate(times_deserialized_seconds):
            start_height = ranges[i][0]
            end_height = ranges[i][1]
            width = end_height-start_height
            ax.bar(start_height+width/2, time, bottom=bottoms[i], width=width, label='Time to deserialize', color='tab:red')
            bottoms[i] += time

        # after all deserialization is done, stair wise check next block and advance to block
        times_check_next_block_done = tryBlockSyncCall.get_times_check_next_block_done()
        times_check_next_block_done_seconds = [time.total_seconds() for time in times_check_next_block_done]

        times_extra_message_advanced_to_block = tryBlockSyncCall.get_times_advanced_to_block()
        times_extra_message_advanced_to_block_seconds = [time.total_seconds() for time in times_extra_message_advanced_to_block]
        
        a = 0
        prev_bottom = bottoms[-1]

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
    