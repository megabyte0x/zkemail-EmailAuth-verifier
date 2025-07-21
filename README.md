# zkEmail EmailAuth Verifier

This is a rust implementation of the zkEmail EmailAuth Verifier.

This verifies the proof for the email whose subject includes the following template: `Sign {string}`

## Setup

1. Generate a proof for the email whose subject includes the following template: `Sign {string}`
2. Edit the `proof_data` in `main.rs` to the proof you generated.
3. Run `cargo run` to verify the proof.
