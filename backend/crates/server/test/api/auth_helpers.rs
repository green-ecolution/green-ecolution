use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use rsa::{
    RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPrivateKey, pkcs8::LineEnding, rand_core::OsRng,
    traits::PublicKeyParts,
};
use secrecy::SecretString;
use serde_json::{Value, json};
use server::configuration::AuthSettings;
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::{TestApp, spawn_app_with_auth};

const KID: &str = "test-key-1";

pub struct AuthHarness {
    pub server: MockServer,
    pub issuer_url: String,
    private_key_pem: String,
}

impl AuthHarness {
    pub async fn start() -> Self {
        let mut rng = OsRng;
        let private = RsaPrivateKey::new(&mut rng, 2048).expect("rsa keygen");
        let public = RsaPublicKey::from(&private);

        let private_key_pem = private
            .to_pkcs1_pem(LineEnding::LF)
            .expect("pem encode")
            .to_string();

        let n = URL_SAFE_NO_PAD.encode(public.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public.e().to_bytes_be());

        let server = MockServer::start().await;
        let realm = "green-ecolution";
        let issuer_url = format!("{}/realms/{realm}", server.uri());

        let jwks = json!({
            "keys": [{
                "kid": KID,
                "kty": "RSA",
                "alg": "RS256",
                "use": "sig",
                "n": n,
                "e": e,
            }]
        });

        Mock::given(method("GET"))
            .and(path(format!(
                "/realms/{realm}/protocol/openid-connect/certs"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks))
            .mount(&server)
            .await;

        Self {
            server,
            issuer_url,
            private_key_pem,
        }
    }

    pub fn auth_settings(&self, enabled: bool) -> AuthSettings {
        AuthSettings {
            enabled,
            issuer_url: self.issuer_url.clone(),
            frontend_client_id: "frontend".to_string(),
            backend_client_id: "backend".to_string(),
            backend_client_secret: SecretString::from("backend-secret".to_string()),
            jwks_refresh_interval_secs: 3600,
            jwks_refresh_timeout_secs: 5,
            default_redirect_url: "http://127.0.0.1/cb".to_string(),
            expected_audience: None,
        }
    }

    pub fn sign_token(&self, claim_overrides: Value) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_secs();

        let mut claims = json!({
            "sub": Uuid::new_v4().to_string(),
            "iss": self.issuer_url,
            "iat": now,
            "exp": now + 300,
            "preferred_username": "tester",
            "email": "test@example.com",
            "realm_access": { "roles": ["tbz"] },
        });

        if let Value::Object(extra) = claim_overrides
            && let Value::Object(base) = &mut claims
        {
            for (k, v) in extra {
                base.insert(k, v);
            }
        }

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(KID.to_string());
        let key = EncodingKey::from_rsa_pem(self.private_key_pem.as_bytes())
            .expect("encoding key from pem");
        encode(&header, &claims, &key).expect("sign token")
    }
}

pub async fn spawn_with_auth() -> (AuthHarness, TestApp) {
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_auth(harness.auth_settings(true)).await;
    (harness, app)
}
