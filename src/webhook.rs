use chrono::Utc;
use reqwest::Method;
use serde_json::Value;
use tokio::task;

use crate::http_client::make_http_request;
use crate::CONFIG;

/// Sends an event payload to the configured webhook URL.
/// This is fire-and-forget: failures are logged but do not block the caller.
pub fn send_webhook(event_data: Value) {
    if !CONFIG.webhook_enabled() || CONFIG.webhook_url().is_empty() {
        return;
    }

    let url = CONFIG.webhook_url();

    task::spawn(async move {
        if let Err(e) = _send_webhook(&url, &event_data).await {
            warn!("Webhook delivery failed: {e}");
        }
    });
}

async fn _send_webhook(url: &str, event_data: &Value) -> Result<(), crate::Error> {
    let request = make_http_request(Method::POST, url)?
        .header("Content-Type", "application/json")
        .header("User-Agent", "Passwarden-Webhook/1.0")
        .header("X-Passwarden-Event", event_data["type"].as_i64().unwrap_or(0).to_string())
        .header("X-Passwarden-Delivery", crate::util::get_uuid())
        .header("X-Passwarden-Timestamp", Utc::now().timestamp().to_string())
        .json(event_data);

    let response = request.send().await?;
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
