### Permanent identities (state of the receiver & sender)

Both sender & receiver modes store their state to `.identity` directory in the directory from which the binary is run. At the moment those are not encrypted (TODO using ~/.ssh/ keys) and the state writes are not fault-tolerant (TODO using speculative write & atomic `mv`).

```sh
./scripts/clear-ids.sh
```

### Running without a network via files only

You use `sender` mode to sign the message from file and make it output it back to some file.
Then you can verify the output with `receiver` mode and, if the signature is valid, output the original message to the file.

```sh
# Sign the message in `./env/data.input` and store the signed block to `./env/data.signed`
./target/debug/audibro sender "0.0.0.0:5555" --input ./env/data.input --output ./env/data.signed

# Verify the signed block  in `./env/data.signed` and if valid write it to `./env/data.output`
./target/debug/audibro receiver "127.0.0.1:5555" --input ./env/data.signed --output ./env/data.output
```
