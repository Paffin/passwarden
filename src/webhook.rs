use chrono::Utc;
use data_encoding::HEXLOWER;
use reqwest::Method;
use ring::hmac;
use serde_json::Value;
use std::sync::LazyLock;
use tokio::sync::Semaphore;
use tokio::task;

use crate::http_client::make_http_request;
use crate::CONFIG;

/// Maximum number of concurrent in-flight webhook deliveries.
static WEBHOOK_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(32));

/// Sends an event payload to the configured webhook URL.
/// This is fire-and-forget: failures are logged but do not block the caller.
pub fn send_webhook(event_data: Value) {
    if !CONFIG.webhook_enabled() {
        return;
    }

    let url = CONFIG.webhook_url();
    if url.is_empty() {
        warn!("WEBHOOK_ENABLED is true but WEBHOOK_URL is not set — skipping delivery");
        return;
    }

    let secret = CONFIG.webhook_secret();

    task::spawn(async move {
        // Acquire semaphore permit to limit concurrent deliveries
        let _permit = match WEBHOOK_SEMAPHORE.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                warn!("Webhook delivery dropped: too many in-flight requests");
                return;
            }
        };

        if let Err(e) = _send_webhook(&url, &secret, &event_data).await {
            warn!("Webhook delivery failed: {e}");
        }
    });
}

async fn _send_webhook(url: &str, secret: &str, event_data: &Value) -> Result<(), crate::Error> {
    let body = serde_json::to_string(event_data).unwrap_or_default();
    let timestamp = Utc::now().timestamp().to_string();
    let delivery_id = crate::util::get_uuid();

    let mut request = make_http_request(Method::POST, url)?
        .header("Content-Type", "application/json")
        .header("User-Agent", "Passwarden-Webhook/1.0")
        .header("X-Passwarden-Event", event_data["type"].as_i64().unwrap_or(0).to_string())
        .header("X-Passwarden-Delivery", &delivery_id)
        .header("X-Passwarden-Timestamp", &timestamp);

    // Compute HMAC-SHA256 signature if a secret is configured
    if !secret.is_empty() {
        let sign_payload = format!("{timestamp}.{body}");
        let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
        let signature = hmac::sign(&key, sign_payload.as_bytes());
        let sig_hex = HEXLOWER.encode(signature.as_ref());
        request = request.header("X-Passwarden-Signature", format!("sha256={sig_hex}"));
    }

    let response = request.body(body).send().await?;
    let status = response.status();

    if !status.is_success() {
        warn!("Webhook endpoint returned HTTP {status}");
    } else {
        debug!("Webhook delivered successfully to {url}");
    }

    Ok(())
}

/// Build event JSON payload for webhook delivery
pub fn build_event_payload(
    event_type: i32,
    user_uuid: Option<&str>,
    org_uuid: Option<&str>,
    cipher_uuid: Option<&str>,
    collection_uuid: Option<&str>,
    group_uuid: Option<&str>,
    member_uuid: Option<&str>,
    acting_user_uuid: Option<&str>,
    ip_address: Option<&str>,
) -> Value {
    json!({
        "type": event_type,
        "date": crate::util::format_date(&Utc::now().naive_utc()),
        "userId": user_uuid,
        "organizationId": org_uuid,
        "cipherId": cipher_uuid,
        "collectionId": collection_uuid,
        "groupId": group_uuid,
        "organizationUserId": member_uuid,
        "actingUserId": acting_user_uuid,
        "ipAddress": ip_address,
    })
}
