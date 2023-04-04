use ethers_core::k256::elliptic_curve::rand_core::{self, CryptoRng, CryptoRngCore, RngCore};
use ethers_core::k256::sha2::Sha256;
pub use rsa::{errors::Error as RSAError, Oaep, PublicKey, RsaPrivateKey, RsaPublicKey};

const ENCRYPTING_CHUNK_SIZE2048PUB_KEY: usize = 126;
const ENCRYPTED_CHUNK_SIZE2048PUB_KEY: usize = 256;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to encrypt with public key: {0}")]
    FailedToEncryptWithPublicKey(RSAError),
    #[error("failed to decrypt with private key: {0}")]
    FailedToDecryptWithPrivateKey(RSAError),
    #[error("invalid chunk size: {0}")]
    InvalidChunkSize(usize),
}

#[derive(Default, Clone)]
pub struct EncryptionResult {
    pub encoded_msg: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Default, Clone)]
pub struct DecryptionResult {
    pub decrypted_msg: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encode_by_chunks(
    msg: Vec<u8>,
    pub_key: RsaPublicKey,
    nonce: Vec<u8>,
) -> Result<EncryptionResult, Error> {
    let mut msg_buffer = msg;
    let result = &mut EncryptionResult::default();
    let mut rng = Noncer::new(rand::thread_rng(), nonce);

    while !msg_buffer.is_empty() {
        let mut chunk = msg_buffer.to_vec();

        if chunk.len() >= ENCRYPTING_CHUNK_SIZE2048PUB_KEY {
            chunk = chunk[..ENCRYPTING_CHUNK_SIZE2048PUB_KEY].to_vec();
            msg_buffer = msg_buffer[ENCRYPTING_CHUNK_SIZE2048PUB_KEY..].to_vec();
        } else {
            msg_buffer = Vec::new();
        }

        result.encoded_msg.append(
            &mut pub_key
                .encrypt(&mut rng, Oaep::new::<Sha256>(), &chunk[..])
                .map_err(Error::FailedToEncryptWithPublicKey)?,
        );
    }

    result.nonce = rng.nonce;
    Ok(result.clone())
}

pub fn decode_by_chunks(msg: Vec<u8>, private_key: RsaPrivateKey) -> Result<Vec<u8>, Error> {
    let mut msg_buffer = msg;
    let mut decrypted_msg: Vec<u8> = Vec::new();

    while !msg_buffer.is_empty() {
        if msg_buffer.len() < ENCRYPTED_CHUNK_SIZE2048PUB_KEY {
            Err(Error::InvalidChunkSize(msg_buffer.len()))?
        }

        let chunk = msg_buffer[..ENCRYPTED_CHUNK_SIZE2048PUB_KEY].to_vec();
        msg_buffer = msg_buffer[ENCRYPTED_CHUNK_SIZE2048PUB_KEY..].to_vec();

        decrypted_msg.append(
            &mut private_key
                .decrypt(Oaep::new::<Sha256>(), chunk.as_slice())
                .map_err(Error::FailedToDecryptWithPrivateKey)?
                .to_vec(),
        );
    }

    Ok(decrypted_msg)
}

/// The Noncer type is implement required for the RSA encryption random fill bytes array
/// filling. After the fill_bytes function call the nonce is stored in the Noncer body
#[derive(Clone)]
pub struct Noncer<R: CryptoRngCore> {
    pub true_rng: R,
    pub nonce: Vec<u8>,
}

impl<R: CryptoRngCore> Noncer<R> {
    fn new(true_rng: R, nonce: Vec<u8>) -> Self {
        Self { true_rng, nonce }
    }
}

impl<R: CryptoRngCore> CryptoRng for Noncer<R> {}

impl<R: CryptoRngCore> RngCore for Noncer<R> {
    fn next_u32(&mut self) -> u32 {
        unimplemented!();
    }

    fn next_u64(&mut self) -> u64 {
        unimplemented!();
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        if !self.nonce.is_empty() {
            dest.copy_from_slice(self.nonce.as_slice());
            return;
        }

        self.true_rng.fill_bytes(dest);
        self.nonce = dest.to_vec()
    }

    fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), rand_core::Error> {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use crate::rsa::{decode_by_chunks, encode_by_chunks};
    use rsa::{RsaPrivateKey, RsaPublicKey};

    #[tokio::test]
    async fn happy_path() {
        let bits = 2048;

        let private_key =
            RsaPrivateKey::new(&mut rand::thread_rng(), bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&private_key);

        let encode_message = "hello world";

        let encode_result =
            encode_by_chunks(encode_message.as_bytes().to_vec(), pub_key, Vec::new()).unwrap();
        let decode_result = decode_by_chunks(encode_result.encoded_msg, private_key).unwrap();

        assert_eq!(
            decode_result,
            encode_message.as_bytes(),
            "source message isn't eq to the result message"
        )
    }

    #[tokio::test]
    async fn custom_nonce() {
        let bits = 2048;

        let private_key =
            RsaPrivateKey::new(&mut rand::thread_rng(), bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&private_key);

        let encode_message = "hello world";

        let encode_result1 = encode_by_chunks(
            encode_message.as_bytes().to_vec(),
            pub_key.clone(),
            Vec::new(),
        )
        .unwrap();

        let encode_result2 = encode_by_chunks(
            encode_message.as_bytes().to_vec(),
            pub_key.clone(),
            encode_result1.nonce.clone(),
        )
        .unwrap();

        assert_eq!(
            encode_result1.encoded_msg.clone(),
            encode_result2.encoded_msg.clone(),
            "encoded messages with the same nonce aren't the same"
        );

        assert_eq!(
            encode_result1.nonce.clone(),
            encode_result2.nonce.clone(),
            "nonces are different"
        );
    }

    #[tokio::test]
    async fn with_different_nonce() {
        let bits = 2048;

        let private_key =
            RsaPrivateKey::new(&mut rand::thread_rng(), bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&private_key);

        let encode_message = "hello world";

        let encode_result1 = encode_by_chunks(
            encode_message.as_bytes().to_vec(),
            pub_key.clone(),
            Vec::new(),
        )
        .unwrap();

        let encode_result2 = encode_by_chunks(
            encode_message.as_bytes().to_vec(),
            pub_key.clone(),
            Vec::new(),
        )
        .unwrap();

        assert_ne!(
            encode_result1.encoded_msg,
            encode_result2.encoded_msg.clone(),
            "encoded messages without the same nonces are the same"
        );

        assert_ne!(
            encode_result1.nonce, encode_result2.nonce,
            "nonces are the same"
        );
    }
}
