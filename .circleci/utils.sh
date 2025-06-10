# Utility functions for devnet scripts

# Function checking that each node reached a sufficient block height.
check_heights() {
  echo "Checking block heights on all nodes..."
  all_reached=true
  highest_height=0
  for ((node_index = 0; node_index < $((total_validators + total_clients)); node_index++)); do
    port=$((3030 + node_index))
    height=$(curl -s "http://127.0.0.1:$port/$network_name/block/height/latest" || echo "0")
    echo "Node $node_index block height: $height"
    
    # Track highest height for reporting
    if [[ "$height" =~ ^[0-9]+$ ]] && [ $height -gt $highest_height ]; then
      highest_height=$height
    fi
    
    if ! [[ "$height" =~ ^[0-9]+$ ]] || [ $height -lt $min_height ]; then
      all_reached=false
    fi
  done
  
  if $all_reached; then
    echo "✅ SUCCESS: All nodes reached minimum height of $min_height"
    return 0
  else
    echo "⏳ WAITING: Not all nodes reached minimum height of $min_height (highest so far: $highest_height)"
    return 1
  fi
}

# Function checking that nodes created logs on disk.
check_logs() {
  echo "Checking logs for all nodes..."
  all_reached=true
  highest_height=0
  for ((validator_index = 0; validator_index < $total_validators; validator_index++)); do
    if [ ! -s "$log_dir/validator-${validator_index}.log" ]; then
      echo "❌ Test failed! Validator #${validator_index} did not create any logs."
      return 1
    fi
  done
  for ((client_index = 0; client_index < $total_clients; client_index++)); do
    if [ ! -s "$log_dir/client-${client_index}.log" ]; then
      echo "❌ Test failed! Client #${client_index} did not create any logs."
      return 1
    fi
  done

  return 0
}
