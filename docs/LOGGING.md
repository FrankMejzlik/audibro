## **Logging**

For sake of readability we are using a "tagged" output to different files which you can monitor in real time (e.g. using `tail -f`). This effectively emulates a multiple terminals. Feel free to open multiple terminal (or use e.g. `tmux` to split into panes) and monitor whatever you're interested in.

### `tmux` one-liners

```sh
# TODO: write a tmux one-liner that splits the panes for you and runs the belowmentioned commands to live monitor the logs
```

### Sender

#### **Supported logs:**

```sh
# General log
tail -f ./env/sender-alice/logs/output.log
# ---
# The main sender loop
tail -f ./env/sender-alice/logs/sender.log
# The UTF-8 repre of broadcasted messages
tail -f ./env/sender-alice/logs/broadcasted.log
# The state of key layers
tail -f ./env/sender-alice/logs/block_signer.log
# ---
# The task managing the requests from receivers
tail -f ./env/sender-alice/logs/registrator_task.log
# The list of active subscribers
tail -f ./env/sender-alice/logs/subscribers.log
```

### Receiver

#### **Supported logs**

```sh
# General log
tail -f ./env/receiver-bob/logs/output.log
# ---
# The main sender loop
tail -f ./env/receiver-bob/logs/receiver.log
# The UTF-8 repre of valid received messages
tail -f ./env/receiver-bob/logs/received.log
# The state of the public keys in the keystore
tail -f ./env/receiver-bob/logs/block_verifier.log
tail -f ./env/receiver-bob/logs/delivery_queues.log
# ---
# The task sending periodic heartbeats to the sender
tail -f ./env/receiver-bob/logs/heartbeat_task.log
# The inner state of fragmented block receiver
tail -f ./env/receiver-bob/logs/fragmented_blocks.log
```
