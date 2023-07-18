## Permanent identities (state of the receiver & sender)

Both sender & receiver modes store their state to `.identity` directory in the directory from which the binary is run. At the moment those are not encrypted and the state writes are not fault-tolerant.

> When you change any configuration related to the scheme, the existing identities have to be purged since they are likely not in the appropriate sizes.

```sh
./scripts/clear-ids.sh
```

## Configuration of the scheme instance

To configure the instance scheme parameters, one should head to `config.rs` and modify the code that is assigned to the alias `SignerInst`. Based on whether `debug` feature is on or not (in `Cargo.toml`) the according set of parameters is used from `PARAMETERS` code block.

This is how the default release configuration looks like:
```rs
/// Size of the hashes in a Merkle tree
const N: usize = 256 / 8;
/// Number of SK segments in signature
const K: usize = 16;
/// Depth of the Merkle tree (without the root layer)
const TAU: usize = 16;

// --- Random generators ---
/// A seedable CSPRNG used for number generation
type CsPrng = ChaCha20Rng;

/// Maximum number of secure signature per one key
const KEY_CHARGES: usize = 16;

// --- Hash function ---
type HashFn = Sha3_256;

const T: usize = 2_usize.pow(TAU as u32);

// The final signer type
pub type SignerInst = HorstSigScheme<N, K, TAU, { TAU + 1 }, T, KEY_CHARGES, CsPrng, HashFn>;
```

## Implementing custom few-time signature scheme

The `SignerInst` alias is assigned the scheme type with its parameters. The signature scheme must implement the [`FtsScheme`](https://gitlab.mff.cuni.cz/mejzlikf/hab/-/blob/master/src/traits.rs#L125) trait. That's it! Once you have that, your signature scheme will work as a drop-in replacement for the bundled-in HORST scheme.