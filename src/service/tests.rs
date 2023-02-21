// use crate::service::{
//     storage::in_memory::MapStorage, types::Participant, waiter::simple::SimpleWaiter, Service,
// };
// use ethers_core::types::{Address, U256};
use rsa::pkcs8::DecodePublicKey;

const PARTICIPANTS_NUMBER: usize = 4;

/// Public keys for the various of scenarios in tests.
const RSA_RAW_PUBLIC_KEYS: [&'static str; PARTICIPANTS_NUMBER] = [
    r#"
-----BEGIN PUBLIC KEY-----
MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgFdE/Dy8pWn8TbasNDguHQF1kplm
6RQrPGa5oHwH89VVtv0JV8Yu60ZjIQoqfHaKynesUR4ecoVgTbPyBUiZXHvl2WEk
viNIa2/gowrNLefx1RCN7mnCGXn2i4OhfKsgl5GRqpeuG6JOI4HStxtQ8Jv6q7qx
yk24kgbJZVp6uTjlAgMBAAE=
-----END PUBLIC KEY-----
"#,
    r#"
-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCBudbpatqjI2AaOB+JNN11T331
c3NPAY8Jz8hrq9/nx114hIqTUrm8qRpZ8U+ZUNemJtDlFd/4DtqHc1yVnoBE1XBu
RzAV5Jh9lPxmg6cZ6Zi3+oK1IgKncWGte6rmqSWZHpUZZcr4o11kqDdMn4Yqtfku
Abn4yal94eJxuRrc3QIDAQAB
-----END PUBLIC KEY-----
"#,
    r#"
-----BEGIN PUBLIC KEY-----
MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgGtADnBsitKFbp8LR7OTmUCpjwku
DIEBxLiwBN3LbG7bSNUkSkemNOfcRyzafkbIW1jycBzIAZ4nyI+/OR4OpMiu4Qfj
CKtTZlwIbH1YOy5dAgK/gp02kwW32WUDrgiqXLAo33WgsyrrnShiyzO8/Hs1W02m
aO4Sfu6FYuKn5mCPAgMBAAE=
-----END PUBLIC KEY-----
"#,
    r#"
-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCSTymVXlp79GOyqFkfg7PW0/jr
g3TUZC8QbrXmH5U9sox55rXpgihwKvt4dGrNhsfQeiRzAF8KFi5iYktH0UFa2TuA
ojqtAfQcol/HsiFgJxhqKPqhf8F0mUaEr60vyWRW4uKvWp+tCDGHmDnHRLxFlWmu
kxfPxATu7oTrNJmGcwIDAQAB
-----END PUBLIC KEY-----
"#,
];

lazy_static::lazy_static! {
    /// Public keys for the various of scenarios in tests.
    static ref RSA_PUBLIC_KEYS: Vec<rsa::RsaPublicKey> = RSA_RAW_PUBLIC_KEYS
            .iter()
            .map(|key| rsa::RsaPublicKey::from_public_key_pem(key).unwrap())
            .collect();
}

// For 5 participants in the room, return keys that are needed for participants
// to encode and decode outputs
// #[tokio::test]
// async fn happy_path() {
//     let token: Address = Address::default();
//     let amount: U256 = U256::from(5);
//     let room_size = PARTICIPANTS_NUMBER;

//     let storage = MapStorage::default();

//     let waiter = SimpleWaiter::new(room_size, storage.clone());

//     let service = Service::new(storage.clone(), waiter, );

//     for i in 0..room_size {
//         let participant = Participant::new(U256::from(i), RSA_PUBLIC_KEYS[i].clone());

//         service
//             .add_participant(&token, &amount, &participant)
//             .await
//             .unwrap();
//     }

//     let rooms = service.create_rooms(&token, &amount).await.unwrap();

//     assert_eq!(rooms.len(), 1, "should be only one room");

//     let room = &rooms[0];

//     for (position, participant) in room.participants.iter().enumerate() {
//         let keys = service.participant_keys(participant).await.unwrap();

//         assert_eq!(
//             keys.len(),
//             room.participants.len() - position,
//             "invalid number of keys",
//         );
//     }
// }
