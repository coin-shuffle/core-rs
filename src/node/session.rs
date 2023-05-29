use std::mem::size_of;

use coin_shuffle_contracts_bindings::shared_types::Utxo;
use ethers_core::abi::{AbiEncode, Hash};
use ethers_core::types::{Address, U256};
use ethers_core::utils::keccak256;

use crate::rsa;
use crate::rsa::{EncryptionResult, Error as RSAError, RsaPrivateKey, RsaPublicKey};
use crate::types::EncryptedOutput;

const ADDRESS_SIZE: usize = size_of::<Address>();
const U256_SIZE: usize = size_of::<U256>();

/// The starting point of the **CoinShuffle** process from `node` (client) side.
///
/// This state is passive until, the RSA public keys of other participatns are passed
/// by [`add_participants_keys`](Session::add_participants_keys) method.
///
/// # Example
///
/// ```ignore
/// use ethers_core::types::Address;
///
/// use coin_shuffle_core::node::Session;
///
/// let utxo = Utxo {
///    amount: 100,
///    token: Address::zero(), // Use zero address for ETH.
///    owner: Address::zero(), // Use your own address.
///    is_spent: false,
/// };
///
/// let output = Address::zero(); // Use your receiver address.
///
/// let rsa_private_key = RsaPrivateKey::new(); // Generate RSA key pair.
///
/// let session = Session::new(utxo, output, rsa_private_key);
///
/// let public_keys = vec![RsaPublicKey::new()]; // Get RSA public keys of other participants.
///
/// let session_with_keys = session.add_participants_keys(public_keys);
///
/// // Now you can start the **shuffle** process.
/// let encrypted_outputs = session_with_keys.shuffle_round(vec![]).unwrap();
///
/// // Other participants will send you their encrypted outputs.
///
///
/// let fully_decrypted_outputs = vec![]; // for example
///
/// let message_to_sign = session_with_keys.message_to_sign(fully_decrypted_outputs);
/// ```
#[derive(Clone, Debug)]
pub struct Session<Output: AbiEncode + Clone> {
    /// Utxo that participant wants to shuffle.
    pub utxo: Utxo,
    /// Output that participant wants to shuffle.
    ///
    /// It could be an Ethereum address or something else, depending on the
    /// implementation of the [`AbiEncode`](crate::types::AbiEncode) trait.
    pub output: Output,
    /// RSA private key of the participant.
    ///
    /// This will be used to decrypt the outputs of other participants. The
    /// public key of this key pair will be send to another participants and
    /// will be used to encrypt the output of the current participant.
    pub rsa_private_key: RsaPrivateKey,
}

impl<Output: AbiEncode + Clone> Session<Output> {
    pub fn new(utxo: Utxo, output: Output, rsa_private_key: RsaPrivateKey) -> Self {
        Self {
            utxo,
            output,
            rsa_private_key,
        }
    }

    /// Provide RSA public keys of other participants to the **shuffle** process
    /// session.
    pub fn add_participants_keys(self, public_keys: Vec<RsaPublicKey>) -> SessionWithKeys<Output> {
        SessionWithKeys::from_keys(self, public_keys)
    }
}

/// Represents the active state of the **CoinShuffle** process from `node` (client) side.
///
/// This state is active after the RSA public keys of other participatns are passed
/// by [`add_participants_keys`](Session::add_participants_keys) method.
#[derive(Clone, Debug)]
pub struct SessionWithKeys<Output: AbiEncode + Clone> {
    /// The session itself.
    pub session: Session<Output>,
    /// RSA public keys of other participants.
    pub public_keys: Vec<RsaPublicKey>,
}

impl<Output: AbiEncode + Clone> SessionWithKeys<Output> {
    /// A private method for [`Session`] to create a new [`SessionWithKeys`]
    /// instance.
    pub(crate) fn from_keys(session: Session<Output>, public_keys: Vec<RsaPublicKey>) -> Self {
        Self {
            session,
            public_keys,
        }
    }

    /// Perform a **shuffle** round.
    ///
    /// Decrypt the outputs of other participants and encrypt the output of the
    /// current one.
    ///
    /// # Errors
    ///
    /// Returns an error if decryption or encryption fails.
    pub fn shuffle_round(
        &mut self,
        encrypted_outputs: Vec<EncryptedOutput>,
    ) -> Result<Vec<EncryptedOutput>, ShuffleRoundError> {
        // TODO(OmegaTymbJIep): validate encoded outputs size

        // Decrypt outputs of other participants.
        let mut outputs = encrypted_outputs
            .into_iter()
            .map(|o| rsa::decrypt_by_chunks(o, &self.session.rsa_private_key))
            .collect::<Result<Vec<EncryptedOutput>, RSAError>>()
            .map_err(ShuffleRoundError::Decryption)?;

        // Add encrypted output of current participant.
        let mut last_nonce = Vec::<u8>::new();
        let mut encypted_output = self.session.output.clone().encode();

        for public_key in self.public_keys.iter() {
            let EncryptionResult { nonce, encoded_msg } =
                rsa::encrypt_by_chunks(encypted_output.clone(), &public_key, last_nonce.clone())
                    .map_err(ShuffleRoundError::Encryption)?;

            last_nonce = nonce;
            encypted_output = encoded_msg;
        }

        outputs.push(encypted_output);

        Ok(outputs)
    }

    /// Provide message that participant should sign.
    ///
    /// # Returns
    ///
    /// The result is a hash of concatenated amount of each input and outputs.
    ///
    /// As we have only one input and all participants have the same amount in input,
    /// we could just hash like this:
    ///
    /// ```ignore
    /// room.amount | output1 | room.amount | output2 | ...
    /// ```
    pub fn message_to_sign(self, outputs: Vec<Address>) -> Hash {
        let mut message = Vec::with_capacity(outputs.len() * (ADDRESS_SIZE + U256_SIZE));

        for output in outputs {
            message.extend_from_slice(&self.session.utxo.amount.encode());
            message.extend_from_slice(&output.as_bytes());
        }

        Hash::from(keccak256(message))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShuffleRoundError {
    #[error("Failed to decrypt output of other participant: {0}")]
    Decryption(rsa::Error),
    #[error("Failed to encrypt output of current participant: {0}")]
    Encryption(rsa::Error),
}
