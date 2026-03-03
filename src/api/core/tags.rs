use rocket::serde::json::Json;
use serde_json::Value;

use crate::{
    api::{EmptyResult, JsonResult, Notify, UpdateType},
    auth::Headers,
    db::{
        models::{Cipher, CipherId, Tag, TagCipher, TagId},
        DbConn,
    },
};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_tags,
        get_tag,
        post_tags,
        post_tag,
        put_tag,
        delete_tag_post,
        delete_tag,
        put_cipher_tags,
        post_cipher_tags,
    ]
}

#[get("/tags")]
async fn get_tags(headers: Headers, conn: DbConn) -> Json<Value> {
    let tags = Tag::find_by_user(&headers.user.uuid, &conn).await;
    let tags_json: Vec<Value> = tags.iter().map(Tag::to_json).collect();

    Json(json!({
      "data": tags_json,
      "object": "list",
      "continuationToken": null,
    }))
}

#[get("/tags/<tag_id>")]
async fn get_tag(tag_id: TagId, headers: Headers, conn: DbConn) -> JsonResult {
    match Tag::find_by_uuid_and_user(&tag_id, &headers.user.uuid, &conn).await {
        Some(tag) => Ok(Json(tag.to_json())),
        _ => err!("Invalid tag", "Tag does not exist or belongs to another user"),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagData {
    pub name: String,
}

#[post("/tags", data = "<data>")]
async fn post_tags(data: Json<TagData>, headers: Headers, conn: DbConn, nt: Notify<'_>) -> JsonResult {
    let data: TagData = data.into_inner();

    let mut tag = Tag::new(headers.user.uuid.clone(), data.name);

    tag.save(&conn).await?;
    nt.send_user_update(UpdateType::SyncSettings, &headers.user, &headers.device.push_uuid, &conn).await;

    Ok(Json(tag.to_json()))
}

#[post("/tags/<tag_id>", data = "<data>")]
async fn post_tag(tag_id: TagId, data: Json<TagData>, headers: Headers, conn: DbConn, nt: Notify<'_>) -> JsonResult {
    put_tag(tag_id, data, headers, conn, nt).await
}

#[put("/tags/<tag_id>", data = "<data>")]
async fn put_tag(
    tag_id: TagId,
    data: Json<TagData>,
    headers: Headers,
    conn: DbConn,
    nt: Notify<'_>,
) -> JsonResult {
    let data: TagData = data.into_inner();

    let Some(mut tag) = Tag::find_by_uuid_and_user(&tag_id, &headers.user.uuid, &conn).await else {
        err!("Invalid tag", "Tag does not exist or belongs to another user")
    };

    tag.name = data.name;

    tag.save(&conn).await?;
    nt.send_user_update(UpdateType::SyncSettings, &headers.user, &headers.device.push_uuid, &conn).await;

    Ok(Json(tag.to_json()))
}

#[post("/tags/<tag_id>/delete")]
async fn delete_tag_post(tag_id: TagId, headers: Headers, conn: DbConn, nt: Notify<'_>) -> EmptyResult {
    delete_tag(tag_id, headers, conn, nt).await
}

#[delete("/tags/<tag_id>")]
async fn delete_tag(tag_id: TagId, headers: Headers, conn: DbConn, nt: Notify<'_>) -> EmptyResult {
    let Some(tag) = Tag::find_by_uuid_and_user(&tag_id, &headers.user.uuid, &conn).await else {
        err!("Invalid tag", "Tag does not exist or belongs to another user")
    };

    tag.delete(&conn).await?;

    nt.send_user_update(UpdateType::SyncSettings, &headers.user, &headers.device.push_uuid, &conn).await;
    Ok(())
}

// Cipher-tag assignment endpoints

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CipherTagsData {
    tag_ids: Vec<TagId>,
}

/// Replace all tags on a cipher with the given set
#[put("/ciphers/<cipher_id>/tags", data = "<data>")]
async fn put_cipher_tags(
    cipher_id: CipherId,
    data: Json<CipherTagsData>,
    headers: Headers,
    conn: DbConn,
    nt: Notify<'_>,
) -> EmptyResult {
    let Some(cipher) = Cipher::find_by_uuid(&cipher_id, &conn).await else {
        err!("Cipher doesn't exist")
    };

    if !cipher.is_write_accessible_to_user(&headers.user.uuid, &conn).await {
        err!("Cipher is not owned by user")
    }

    let data = data.into_inner();

    // Validate all tag IDs belong to this user
    for tag_id in &data.tag_ids {
        if Tag::find_by_uuid_and_user(tag_id, &headers.user.uuid, &conn).await.is_none() {
            err!("Invalid tag", "Tag does not exist or belongs to another user");
        }
    }

    // Remove all existing tags for this cipher
    TagCipher::delete_all_by_cipher(&cipher_id, &conn).await?;

    // Add the new tags
    for tag_id in data.tag_ids {
        let tag_cipher = TagCipher::new(tag_id, cipher_id.clone());
        tag_cipher.save(&conn).await?;
    }

    nt.send_user_update(UpdateType::SyncCipherUpdate, &headers.user, &headers.device.push_uuid, &conn).await;
    Ok(())
}

#[post("/ciphers/<cipher_id>/tags", data = "<data>")]
async fn post_cipher_tags(
    cipher_id: CipherId,
    data: Json<CipherTagsData>,
    headers: Headers,
    conn: DbConn,
    nt: Notify<'_>,
) -> EmptyResult {
    put_cipher_tags(cipher_id, data, headers, conn, nt).await
}
