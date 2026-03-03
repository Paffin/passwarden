use rocket::serde::json::Json;
use rocket::Route;
use serde_json::Value;
use std::str::FromStr;
use uuid::Uuid;
use webauthn_rs::prelude::PasskeyRegistration;
use webauthn_rs_proto::UserVerificationPolicy;

use super::two_factor::webauthn::{RegisterPublicKeyCredentialCopy, WEBAUTHN};
use crate::api::{EmptyResult, JsonResult, PasswordOrOtpData};
use crate::auth::Headers;
use crate::db::{
    models::{TwoFactor, TwoFactorType, WebAuthnCredential, WebAuthnCredentialId},
    DbConn,
};

pub fn routes() -> Vec<Route> {
    routes![
        get_credentials,
        get_attestation_options,
        register_credential,
        register_credential_put,
        delete_credential,
    ]
}

#[get("/webauthn")]
async fn get_credentials(headers: Headers, conn: DbConn) -> JsonResult {
    let credentials = WebAuthnCredential::find_all_by_user(&headers.user.uuid, &conn).await;

    let credentials_json: Vec<Value> = credentials
        .iter()
        .map(|c| {
            json!({
                "id": c.uuid,
                "name": c.name,
                "prfStatus": if c.supports_prf { 1 } else { 0 },
                "supportsPrf": c.supports_prf,
                "object": "webAuthnCredential"
            })
        })
        .collect();

    Ok(Json(json!({
        "object": "list",
        "data": credentials_json,
        "continuationToken": null
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttestationOptionsData {
    master_password_hash: Option<String>,
    otp: Option<String>,
}

#[post("/webauthn/attestation-options", data = "<data>")]
async fn get_attestation_options(data: Json<AttestationOptionsData>, headers: Headers, conn: DbConn) -> JsonResult {
    if !crate::CONFIG.domain_set() {
        err!("`DOMAIN` environment variable is not set. Passkey login disabled")
    }

    let data = data.into_inner();
    let user = &headers.user;

    PasswordOrOtpData {
        master_password_hash: data.master_password_hash,
        otp: data.otp,
    }
    .validate(user, false, &conn)
    .await?;

    let existing_credentials = WebAuthnCredential::find_all_by_user(&user.uuid, &conn).await;
    let exclude_credentials: Vec<_> = existing_credentials
        .iter()
        .filter_map(|c| {
            let passkey: webauthn_rs::prelude::Passkey = serde_json::from_str(&c.credential).ok()?;
            Some(passkey.cred_id().to_owned())
        })
        .collect();

    let (mut challenge, state) = WEBAUTHN.start_passkey_registration(
        Uuid::from_str(&user.uuid).expect("Failed to parse UUID"),
        &user.email,
        user.display_name(),
        Some(exclude_credentials),
    )?;

    // For passkey login, require user verification and discoverable credentials
    if let Some(asc) = challenge.public_key.authenticator_selection.as_mut() {
        asc.user_verification = UserVerificationPolicy::Required;
    }

    // Store challenge state (reuse WebauthnRegisterChallenge type since user is authenticated)
    let type_ = TwoFactorType::WebauthnRegisterChallenge;
    TwoFactor::new(user.uuid.clone(), type_, serde_json::to_string(&state)?).save(&conn).await?;

    let mut challenge_value = serde_json::to_value(challenge.public_key)?;
    challenge_value["status"] = "ok".into();
    challenge_value["errorMessage"] = "".into();
    Ok(Json(challenge_value))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterCredentialData {
    device_response: RegisterPublicKeyCredentialCopy,
    name: String,
    supports_prf: Option<bool>,
    encrypted_user_key: Option<String>,
    encrypted_public_key: Option<String>,
    encrypted_private_key: Option<String>,
    master_password_hash: Option<String>,
    otp: Option<String>,
}

#[post("/webauthn", data = "<data>")]
async fn register_credential(data: Json<RegisterCredentialData>, headers: Headers, conn: DbConn) -> JsonResult {
    let data = data.into_inner();
    let user = &headers.user;

    PasswordOrOtpData {
        master_password_hash: data.master_password_hash,
        otp: data.otp,
    }
    .validate(user, true, &conn)
    .await?;

    // Retrieve and delete the saved challenge state
    let type_ = TwoFactorType::WebauthnRegisterChallenge as i32;
    let state = match TwoFactor::find_by_user_and_type(&user.uuid, type_, &conn).await {
        Some(tf) => {
            let state: PasskeyRegistration = serde_json::from_str(&tf.data)?;
            tf.delete(&conn).await?;
            state
        }
        None => err!("Can't recover challenge"),
    };

    // Verify the credential with webauthn-rs
    let credential = WEBAUTHN.finish_passkey_registration(&data.device_response.into(), &state)?;

    let supports_prf = data.supports_prf.unwrap_or(false);

    // Store in web_authn_credentials table
    let wac = WebAuthnCredential::new(
        user.uuid.clone(),
        data.name,
        serde_json::to_string(&credential)?,
        supports_prf,
        data.encrypted_user_key,
        data.encrypted_public_key,
        data.encrypted_private_key,
    );
    wac.save(&conn).await?;

    Ok(Json(json!({
        "id": wac.uuid,
        "name": wac.name,
        "prfStatus": if wac.supports_prf { 1 } else { 0 },
        "supportsPrf": wac.supports_prf,
        "object": "webAuthnCredential"
    })))
}

#[put("/webauthn", data = "<data>")]
async fn register_credential_put(data: Json<RegisterCredentialData>, headers: Headers, conn: DbConn) -> JsonResult {
    register_credential(data, headers, conn).await
}

#[delete("/webauthn/<uuid>")]
async fn delete_credential(uuid: WebAuthnCredentialId, headers: Headers, conn: DbConn) -> EmptyResult {
    WebAuthnCredential::delete_by_uuid_and_user(&uuid, &headers.user.uuid, &conn).await
}
