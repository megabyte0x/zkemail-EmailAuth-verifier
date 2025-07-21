use crate::groth::verify;
use alloy::{
    dyn_abi::SolType,
    primitives::{hex, U256},
    sol,
};
use std::result::Result::Ok;

// Constants
const Q: U256 = U256::from_limbs([
    0x43e1f593f0000001,
    0x2833e84879b97091,
    0xb85045b68181585d,
    0x30644e72e131a029,
]);

const DOMAIN_FIELDS: usize = 9;
const DOMAIN_BYTES: usize = 255;
const EMAIL_FIELDS: usize = 9;
const EMAIL_BYTES: usize = 256;
const COMMAND_FIELDS: usize = 20;
const COMMAND_BYTES: usize = 605;
const PUBKEY_FIELDS: usize = 17;

sol! {
    #[derive(Debug)]
    struct PubkeyArray {
        uint256[17] values;
    }

    #[derive(Debug)]
    struct Proof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
    }

    #[derive(Debug)]
    struct ProveAndClaimCommand {
        string domain;
        string email;
        string resolver;
        string[] emailParts;
        address owner;
        bytes32 dkimSignerHash;
        bytes32 nullifier;
        uint256 timestamp;
        bytes32 accountSalt;
        bool isCodeEmbedded;
        bytes miscellaneousData;
        bytes proof;
    }
}

pub fn decode_data(data: &str) -> Result<ProveAndClaimCommand, ()> {
    let input = hex::decode(data);

    match input {
        Ok(input) => {
            let decoded = ProveAndClaimCommand::abi_decode(&input);
            match decoded {
                Ok(decoded) => {
                    return Ok(decoded);
                }
                Err(e) => {
                    println!("Error in Decoding {}", e);
                    return Err(());
                }
            }
        }
        Err(e) => {
            println!("Error in Decoding {}", e);
            return Err(());
        }
    }
}

pub fn decode_proof(proof: &str) -> Result<Proof, ()> {
    let input = hex::decode(proof);
    match input {
        Ok(input) => {
            let decoded = Proof::abi_decode(&input);
            match decoded {
                Ok(value) => {
                    return Ok(value);
                }
                Err(e) => {
                    return Err(());
                }
            }
        }
        Err(e) => {
            return Err(());
        }
    }
}

pub fn _verify_email(email_parts: Vec<String>, email: String) -> bool {
    // Compose email from parts by joining with dots
    let mut composed_email = String::new();
    for (i, part) in email_parts.iter().enumerate() {
        composed_email.push_str(part);
        if i < email_parts.len() - 1 {
            composed_email.push('.');
        }
    }

    // Convert both strings to bytes for comparison
    let composed_bytes = composed_email.as_bytes();
    let email_bytes = email.as_bytes();

    // Ensure both have the same length
    if composed_bytes.len() != email_bytes.len() {
        return false;
    }

    // Compare byte by byte with special rule: '@' in email should match '$' in composed
    for i in 0..email_bytes.len() {
        let email_byte = email_bytes[i];
        let composed_byte = composed_bytes[i];

        if email_byte == b'@' {
            // '@' in email should correspond to '$' in composed email
            if composed_byte != b'$' {
                return false;
            }
        } else if email_byte != composed_byte {
            // All other characters should match exactly
            return false;
        }
    }

    true
}

pub fn _check_if_less_than_q(proof: &Proof) -> bool {
    if proof.pA[0] < Q
        && proof.pA[1] < Q
        && proof.pB[0][0] < Q
        && proof.pB[0][1] < Q
        && proof.pB[1][0] < Q
        && proof.pB[1][1] < Q
        && proof.pC[0] < Q
        && proof.pC[1] < Q
    {
        return true;
    }
    return false;
}

/**
 * @notice Convert proof string of bytes to a [u8]
 */
pub fn _convert_proof(proof: &str) -> Result<Vec<u8>, hex::FromHexError> {
    return hex::decode(proof);
}

/// Packs bytes into 31-byte field elements (little-endian), padding with zeros if needed.
pub fn pack_bytes_to_fields(bytes: &[u8], padded_size: usize) -> Vec<U256> {
    let remain = padded_size % 31;
    let mut num_fields = (padded_size - remain) / 31;
    if remain > 0 {
        num_fields += 1;
    }
    let mut fields = Vec::with_capacity(num_fields);
    for i in 0..num_fields {
        let mut field = U256::ZERO;
        for j in 0..31 {
            let idx = i * 31 + j;
            if idx >= padded_size {
                break;
            }
            let byte_val = if idx < bytes.len() { bytes[idx] } else { 0u8 };
            field += U256::from(byte_val) << (8 * j);
        }
        fields.push(field);
    }
    fields
}

// Generates the expected command string for a given owner address
pub fn get_expected_command(message: &str) -> String {
    format!("Sign {}", message)
    // change this to equivalent of abi.encoodePacked() in solidity
}

pub fn _build_pub_signals(command: &ProveAndClaimCommand) -> Vec<u8> {
    let mut pub_signals = vec![U256::ZERO; 60];

    // 1. domain_name (9 fields)
    let domain_fields = pack_bytes_to_fields(command.domain.as_bytes(), DOMAIN_BYTES);
    // println!("Domain Fields: {:#?}", domain_fields);
    for i in 0..DOMAIN_FIELDS {
        pub_signals[i] = domain_fields.get(i).cloned().unwrap_or(U256::ZERO);
    }

    // 2. public_key_hash (1 field)
    // println!(
    //     "DKIM Signer Hash in the U256: {:#?}",
    //     U256::from_be_bytes(command.dkimSignerHash.0)
    // );
    pub_signals[DOMAIN_FIELDS] = U256::from_be_bytes(command.dkimSignerHash.0);

    // 3. email_nullifier (1 field) - using accountSalt as nullifier for now
    // println!(
    //     "Nullifier in U256: {:#?}",
    //     U256::from_be_bytes(command.nullifier.0)
    // );
    pub_signals[DOMAIN_FIELDS + 1] = U256::from_be_bytes(command.nullifier.0);

    // 4. timestamp (1 field)
    // println!("Timestamp: {}", command.timestamp);
    pub_signals[DOMAIN_FIELDS + 2] = command.timestamp;

    // 5. masked_command (20 fields)
    let expected_command = get_expected_command(&command.resolver);
    let command_fields = pack_bytes_to_fields(expected_command.as_bytes(), COMMAND_BYTES);
    // println!("Command Fields: {:?}", command_fields);¯
    for i in 0..COMMAND_FIELDS {
        pub_signals[DOMAIN_FIELDS + 3 + i] = command_fields.get(i).cloned().unwrap_or(U256::ZERO);
    }

    // 6. account_salt (1 field)
    // println!(
    //     "Account salt in U256: {}",
    //     U256::from_be_bytes(command.accountSalt.0)
    // );
    pub_signals[DOMAIN_FIELDS + 3 + COMMAND_FIELDS] = U256::from_be_bytes(command.accountSalt.0);

    // 7. is_code_exist (1 field)
    pub_signals[DOMAIN_FIELDS + 3 + COMMAND_FIELDS + 1] = if command.isCodeEmbedded {
        U256::from(1)
    } else {
        U256::ZERO
    };

    // 8. pubkey (17 fields) - decode from miscellaneousData
    let pubkey_offset = DOMAIN_FIELDS + 3 + COMMAND_FIELDS + 2;
    let pubkey = PubkeyArray::abi_decode(&command.miscellaneousData);
    match pubkey {
        Ok(v) => {
            for i in 0..PUBKEY_FIELDS {
                pub_signals[pubkey_offset + i] = v.values[i];
            }
            // println!("Pubkey: {:#?}", v.values);
        }
        Err(e) => {
            println!("Error in retrieving pubkey: {}", e)
        }
    }

    // 9. email_address (9 fields)
    let email_fields = pack_bytes_to_fields(command.email.as_bytes(), EMAIL_BYTES);
    // println!("Email fields: {:#?}", email_fields);
    let email_offset = pubkey_offset + PUBKEY_FIELDS;
    for i in 0..EMAIL_FIELDS {
        pub_signals[email_offset + i] = email_fields.get(i).cloned().unwrap_or(U256::ZERO);
    }

    println!("Pub Signal before conversion: {:#?}", pub_signals);

    // Convert to bytes using Alloy's efficient encoding
    pub_signals
        .iter()
        .flat_map(|x| x.to_be_bytes::<32>())
        .collect()
}

pub fn is_valid_proof(data: &str) -> bool {
    let decoded_data = decode_data(data);

    match decoded_data {
        Ok(value) => {
            let decoded_proof = decode_proof(&value.proof.to_string());
            match decoded_proof {
                Ok(value) => {
                    let result = _check_if_less_than_q(&value);
                    if !result {
                        return false;
                    }
                }
                Err(e) => return false,
            }

            let email_parts = value.emailParts.clone();
            let email = value.email.clone();

            // TODO: implement email verification

            if _verify_email(email_parts, email) {
                return false;
            }

            let proof = match _convert_proof(&value.proof.to_string()) {
                Ok(bytes) => bytes,
                Err(e) => {
                    println!("Failed to convert proof: {}", e);
                    return false;
                }
            };

            let pub_signal = _build_pub_signals(&value);
            // println!("Pub Signal after conversion: {:#?}", pub_signal);

            let result = verify(&proof, &pub_signal);
            match result {
                Ok(val) => {
                    if !val {
                        return false;
                    }
                }
                Err(e) => return false,
            }
        }
        Err(e) => {
            return false;
        }
    }

    return true;
}
