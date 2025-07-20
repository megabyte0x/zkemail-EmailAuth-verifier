use alloy::{
    dyn_abi::SolType,
    primitives::{hex, Address, Bytes, U256},
    sol,
};
use std::alloc::Global;
use std::result::Result::Ok;

type Bytes32 = sol! { bytes32 };

pub struct EmailParts {
    pub part_1: String,
    pub part_2: String,
}

pub struct ProveAndClaimCommands {
    pub domain: String,
    pub email: String,
    pub resolver: String,
    pub email_parts: EmailParts,
    pub address: Address,
    pub dkim_signer_hash: Bytes32,
    pub nullifier: Bytes32,
    pub timestamp: U256,
    pub account_salt: Bytes32,
    pub is_code_embedded: bool,
    pub miscellaneous_data: Bytes,
    pub proof: Bytes,
}

sol! {
    #[derive(Debug)]
    struct Proof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
    }

    #[derive(Debug)]
    struct ProveAndClaimCommand {
        /// @notice The domain part of the email address (e.g., "gmail.com")
        /// @dev Used to identify the email provider and corresponding DKIM public key for verification
        string domain;
        /// @notice The complete email address (e.g., "user@gmail.com")
        /// @dev This is the email address being claimed, which will correspond to the ENS subdomain
        string email;
        /// @notice The resolver ENS name for the ENS name
        /// @dev This ENS name is used to resolve the ENS name to an Ethereum address
        string resolver;
        /// @notice The parts of the email address dot separated (e.g., ["user", "gmail", "com"])
        /// @dev Used to verify the email address
        string[] emailParts;
        /// @notice The Ethereum address that will own the claimed ENS name
        /// @dev This address becomes the owner of the ENS name derived from the email address
        address owner;
        /// @notice Hash of the RSA public key used for DKIM signature verification
        /// @dev This hash uniquely identifies the DKIM public key and ensures the email's authenticity
        bytes32 dkimSignerHash;
        /// @notice A unique identifier used to prevent replay attacks
        /// @dev This nullifier ensures that each email can only be used once for claiming an ENS name
        bytes32 nullifier;
        /// @notice The timestamp from the email header, or 0 if not supported
        /// @dev Some email providers (like Outlook) don't sign timestamps, so this field may be 0
        uint256 timestamp;
        /// @notice Account salt for additional privacy.
        /// @dev Used to hide email address on-chain. Which is not relavant here.
        bytes32 accountSalt;
        /// @notice Indicates whether the verification code is embedded in the email
        /// @dev Used in proof verification
        bool isCodeEmbedded;
        /// @notice Additional data for future compatibility and flexibility
        /// @dev This field can contain DNSSEC proof data, additional verification parameters,
        ///      or any other data required by specific verifier implementations. Can be 0x0 if unused.
        bytes miscellaneousData;
        /// @notice The zero-knowledge proof that validates all fields in this struct
        /// @dev Contains the proof compatible with verifier
        bytes proof;
    }
}

// const Q: U256 =
// U256::from(21_888_242_871_839_275_222_246_405_745_257_275_088_696_311_157_297_823_662_689_037_894_645_226_208_583);

// const DOMAIN_FIELDS = U256::from(9);
// const DOMAIN_BYTES = U256::from(255);
// const EMAIL_FIELDS = U256::from(9);
// const EMAIL_BYTES = U256::from(256);
// const COMMAND_FIELDS = U256::from(20);
// const COMMAND_BYTES = U256::from(605);
// const PUBKEY_FIELDS = U256::from(17);

pub fn decode_data(data: &str) -> Result<ProveAndClaimCommand, ()> {
    let input = hex::decode(data);

    match input {
        Ok(input) => {
            let decoded = ProveAndClaimCommand::abi_decode(&input);
            match decoded {
                Ok(decoded) => {
                    println!("{:?}", decoded);

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
                Ok(decoded) => {
                    return Ok(decoded);
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

pub fn _check_if_less_than_q(proof: &Proof) -> bool {
    if proof.pA[0] < Q {
        return false;
    }
    return true;
}

pub fn _verify_email_parts(email_parts: Vec<String, Global>, email: String) -> bool {
    return true;
}

pub fn isValid(data: &str) -> bool {
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
            let verify_email_parts = _verify_email_parts(value.emailParts, value.email);
            if !verify_email_parts {
                return false;
            }
        }
        Err(e) => {}
    }

    return true;
}
