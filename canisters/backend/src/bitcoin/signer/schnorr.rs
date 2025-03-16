use ic_cdk::api::management_canister::schnorr::{
    SignWithSchnorrArgument, SignWithSchnorrResponse, sign_with_schnorr,
};

use crate::state::read_config;

pub fn mock_schnorr_signature() {}

pub async fn schnorr_sign(
    message: Vec<u8>,
    derivation_path: Vec<Vec<u8>>,
) -> SignWithSchnorrResponse {
    let key_id = read_config(|config| config.schnorrkeyid());

    sign_with_schnorr(SignWithSchnorrArgument {
        message,
        derivation_path,
        key_id,
    })
    .await
    .unwrap()
    .0
}
