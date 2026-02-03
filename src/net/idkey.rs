use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

#[allow(dead_code)]
pub struct IdKey {
    x: String,
    y: String,
    d: String,
    signing_key: SigningKey
}

impl IdKey {
    pub fn get(&self) -> std::string::String { format!("idkey.{}.{}.{}", self.x, self.y, self.d) }
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let public_point = verifying_key.to_encoded_point(false);
        let x: String = URL_SAFE_NO_PAD.encode(public_point.x().unwrap());
        let y: String = URL_SAFE_NO_PAD.encode(public_point.y().unwrap());

        let private_bytes = signing_key.to_bytes();
        let d: String = URL_SAFE_NO_PAD.encode(private_bytes);
        IdKey { x, y, d, signing_key }
    }
}