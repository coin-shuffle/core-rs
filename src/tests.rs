use ethers_core::types::U256;
use lazy_static::lazy_static;
use rsa::RsaPrivateKey;

lazy_static! {
    static ref PARTICIPANTS: Vec<U256> = {
        vec![
            U256::from(0x1u64),
            U256::from(0x2u64),
            U256::from(0x3u64),
            U256::from(0x4u64),
        ]
    };
    static ref PRIVATE_KEYS: Vec<RsaPrivateKey> = {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let mut keys = Vec::new();
        for _ in PARTICIPANTS.iter() {
            let key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
            keys.push(key);
        }
        keys
    };
}
