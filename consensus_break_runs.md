# Runs of the majority/minority tests

All fail logs are sent to Lukasz as archives named:

<num-of-validators>_val_<num-of-bad-validators>_bad_<types-of-possible-problems-in-the-network>.tar.gz

Examples:
16_val_14_bad_spikes.tar.gz
16_val_16_bad_lan_wan_spikes.tar.gz

## Machine type m5.2xlarge ≈ CI pipeline.parameters.twoxlarge

### Default script with 4 being impedded out of 20 validators FAIL

#### 53af408e43f91dcb5373ce3e3664d4d2743c26a8

Happened once:

```
Waited 300 seconds so far...
❌ Test failed! Consensus version did not stabilize within 5 minutes.
🚨 Cleaning up 22 process(es)…
```

Happened multiple times, gave the logs of one of them to Lukasz

It maintains connection (all validators are connected),
it advances blocks - reached blocks over 60 all 3 different times, but when tring to deploy something the transaction is not found.
No errors related to the transaction in the logs, but the transaction was sent (in the logs can be seen).

### Default script with 10 being impedded out of 19 validators FAIL

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Seems with 19 validators it behaves (on this machine) as with 20 - it maintains connection (all validators are connected),
it advances blocks - reached block 97, but when tring to deploy something the transaction is not found. No errors in the logs.

### Default script with 10 being impedded out of 18 validators FAIL

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Seems with 19 validators it behaves (on this machine) as with 20/19 - it maintains connection (all validators are connected),
it advances blocks - reached block 93, but when tring to deploy something the transaction is not found. No errors in the logs.

### Default script with 10 being impedded out of 17 validators PASS

#### 53af408e43f91dcb5373ce3e3664d4d2743c26a8

Pass!

### Default script with 8 being impedded out of 16 validators PASS

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

I think 16 validators is the most stable on this machine type, rarely has the problem with lost transactions like with more validators, so mainly trying to break this one.

Pass!

### Default script with 14 being impedded out of 16 validators PASS

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Pass!

### Default script with 4 being impedded WITH ONLY SPIKES out of 16 validators PASS

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Pass!

### Default script with 1 being with bad-network and 3 being impedded with spikes out of 16 validators PASS

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Modified the script to be able to do different types of problems for specific ranges in the smae run.
Pass!

### Default script with 2 being with bad-network and 4 being impedded with spikes out of 16 validators PASS

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Modified the script to be able to do different types of problems for specific ranges in the smae run.
Pass!

### Default script with 2 being with bad-network and 6 being impedded spikes out of 16 validators FAIL

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Network is ok, but transactions are lost (no errors in the logs), like with the more than 16 validators.

### Default script with 8 being impedded WITH ONLY SPIKES out of 16 validators FAIL

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Network couldn't be advanced with all validators being in some kind of network impediment

### Default script with 14 being impedded WITH ONLY SPIKES out of 16 validators FAIL

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Network couldn't be advanced with all validators being in some kind of network impediment

### Default script with 16 being impedded out of 16 validators FAIL

#### 2a286a8f2455eb9fa26faf48c92d70dd0d663001

Network couldn't be advanced with all validators being in some kind of network impediment

### Default script with 4 being impedded out of 15 validators PASS

#### 53af408e43f91dcb5373ce3e3664d4d2743c26a8

Pass!
