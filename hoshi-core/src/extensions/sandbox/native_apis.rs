use rquickjs::Function;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::debug;
use tracing::warn;

use super::{error_json, FetchRequest, HeadlessRequest, MAX_SLEEP_MS};
use crate::extensions::html_query;

pub(super) fn register_native_apis(
    ctx: &rquickjs::Ctx<'_>,
    headless_available: bool,
    req_tx: Arc<std::sync::mpsc::SyncSender<HeadlessRequest>>,
    fetch_tx: Arc<std::sync::mpsc::SyncSender<FetchRequest>>,
    state_map: Arc<Mutex<HashMap<String, Value>>>,
    extension_id: String,
) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    globals.set(
        "__native_log",
        Function::new(ctx.clone(), {
            let extension_id = extension_id.clone();
            move |msg: String| {
                debug!(target: "sandbox_js", extension = %extension_id, "{}", msg);
                Ok::<(), rquickjs::Error>(())
            }
        })?,
    )?;

    globals.set("__native_fetch", Function::new(ctx.clone(), {
        let fetch_tx = Arc::clone(&fetch_tx);
        move |url: String, method: String, headers_json: String, body: String| -> rquickjs::Result<String> {
            let headers: HashMap<String, String> =
                serde_json::from_str(&headers_json).unwrap_or_default();
            let body = if body.is_empty() { None } else { Some(body) };

            let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel::<String>(1);

            if fetch_tx.send(FetchRequest { url, method, headers, body, reply: reply_tx }).is_err() {
                warn!("Fetch channel closed unexpectedly");
                return Ok(error_json("fetch channel closed".into()));
            }

            Ok(reply_rx
                .recv_timeout(std::time::Duration::from_secs(30))
                .unwrap_or_else(|_| {
                    warn!("Fetch timed out");
                    error_json("fetch timeout".into())
                }))
        }
    })?)?;

    globals.set("__native_sleep", Function::new(ctx.clone(), |ms: u64| {
        std::thread::sleep(std::time::Duration::from_millis(ms.min(MAX_SLEEP_MS)));
        Ok::<(), rquickjs::Error>(())
    })?)?;

    globals.set("__native_html_query", Function::new(ctx.clone(),
     |html: String, selector: String| -> rquickjs::Result<String> {
         use scraper::Html;
         let document = Html::parse_document(&html);

         let results = html_query::execute_selector(&document, &selector);

         Ok(serde_json::to_string(&results).unwrap_or_default())
     },
    )?)?;

    globals.set("__headless_available", headless_available)?;

    globals.set(
        "__native_headless_sync",
        Function::new(
            ctx.clone(),
            move |url: String, options_json: String| -> rquickjs::Result<String> {
                let options = serde_json::from_str(&options_json).unwrap_or_default();
                let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel::<String>(1);

                debug!(url = %url, "Native headless sync called from sandbox");

                if req_tx
                    .send(HeadlessRequest {
                        url,
                        options,
                        reply: reply_tx,
                    })
                    .is_err()
                {
                    warn!("Headless channel closed unexpectedly");
                    return Ok(error_json("headless channel closed".into()));
                }

                Ok(
                    reply_rx
                        .recv_timeout(std::time::Duration::from_secs(15))
                        .unwrap_or_else(|_| {
                            warn!("Headless fetch timed out");
                            error_json("headless timeout".into())
                        }),
                )
            },
        )?,
    )?;

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_get",
            Function::new(
                ctx.clone(),
                move |key: String| -> rquickjs::Result<String> {
                    let guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                    let value = guard.get(&key).cloned().unwrap_or(Value::Null);
                    Ok(
                        serde_json::to_string(&value)
                            .unwrap_or_else(|_| "null".to_string()),
                    )
                },
            )?,
        )?;
    }

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_set",
            Function::new(
                ctx.clone(),
                move |key: String, json_val: String| -> rquickjs::Result<()> {
                    let value: Value =
                        serde_json::from_str(&json_val).unwrap_or(Value::Null);
                    let mut guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                    guard.insert(key, value);
                    Ok(())
                },
            )?,
        )?;
    }

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_delete",
            Function::new(ctx.clone(), move |key: String| -> rquickjs::Result<()> {
                let mut guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                guard.remove(&key);
                Ok(())
            })?,
        )?;
    }

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_keys",
            Function::new(ctx.clone(), move || -> rquickjs::Result<String> {
                let guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                let keys: Vec<&String> = guard.keys().collect();
                Ok(serde_json::to_string(&keys).unwrap_or_else(|_| "[]".to_string()))
            })?,
        )?;
    }

    {
        let state_map = Arc::clone(&state_map);
        globals.set("__native_state_has", Function::new(ctx.clone(), {
            let state_map = Arc::clone(&state_map);
            move |key: String| -> rquickjs::Result<bool> {
                let guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                Ok(guard.contains_key(&key))
            }
        })?)?;
    }

    globals.set("__native_crypto_hash", Function::new(ctx.clone(),
        |algo: String, data: String| -> rquickjs::Result<String> {
          use sha2::{Sha256, Sha512, Digest};
          use sha1::Sha1;
          use md5::Md5;
          let b = data.as_bytes();
          let out = match algo.as_str() {
              "md5"    => { let mut h = Md5::new();    h.update(b); hex::encode(h.finalize()) }
              "sha1"   => { let mut h = Sha1::new();   h.update(b); hex::encode(h.finalize()) }
              "sha256" => { let mut h = Sha256::new(); h.update(b); hex::encode(h.finalize()) }
              "sha512" => { let mut h = Sha512::new(); h.update(b); hex::encode(h.finalize()) }
              _        => return Ok(error_json(format!("unknown hash algo: {}", algo))),
          };
          Ok(out)
        },
    )?)?;

    globals.set("__native_crypto_hmac", Function::new(ctx.clone(),
        |algo: String, key_hex: String, data: String| -> rquickjs::Result<String> {
          use hmac::{Hmac, Mac};
          use sha2::{Sha256, Sha512};
          use sha1::Sha1;
          let key = match hex::decode(&key_hex) {
              Ok(k) => k,
              Err(e) => return Ok(error_json(format!("hmac: bad key hex: {}", e))),
          };
          let out = match algo.as_str() {
              "sha1" => {
                  let mut m = Hmac::<Sha1>::new_from_slice(&key).map_err(|_e| rquickjs::Error::Unknown)?;
                  m.update(data.as_bytes()); hex::encode(m.finalize().into_bytes())
              }
              "sha256" => {
                  let mut m = Hmac::<Sha256>::new_from_slice(&key).map_err(|_| rquickjs::Error::Unknown)?;
                  m.update(data.as_bytes()); hex::encode(m.finalize().into_bytes())
              }
              "sha512" => {
                  let mut m = Hmac::<Sha512>::new_from_slice(&key).map_err(|_| rquickjs::Error::Unknown)?;
                  m.update(data.as_bytes()); hex::encode(m.finalize().into_bytes())
              }
              _ => return Ok(error_json(format!("unknown hmac algo: {}", algo))),
          };
          Ok(out)
        },
    )?)?;

    globals.set("__native_crypto_aes", Function::new(ctx.clone(),
     |op: String, key_hex: String, data_hex: String,
      iv_hex: Option<String>, mode: String| -> rquickjs::Result<String> {
         use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, BlockDecryptMut, KeyIvInit};
         use cbc::{Encryptor, Decryptor};
         use aes::{Aes128, Aes256};
         use aes_gcm::{Aes128Gcm, Aes256Gcm, Nonce, aead::{Aead, KeyInit as GcmKeyInit}};

         let key = match hex::decode(&key_hex) {
             Ok(k) => k, Err(e) => return Ok(error_json(format!("aes: bad key hex: {}", e))),
         };
         let data = match hex::decode(&data_hex) {
             Ok(d) => d, Err(e) => return Ok(error_json(format!("aes: bad data hex: {}", e))),
         };

         if mode == "gcm" {
             let iv = match iv_hex.as_deref() {
                 None => return Ok(error_json("aes-gcm: iv (12-byte nonce) is required and must not be reused".into())),
                 Some(iv_hex) => match hex::decode(iv_hex) {
                     Ok(v) => v,
                     Err(e) => return Ok(error_json(format!("aes: bad iv hex: {}", e))),
                 },
             };
             if iv.len() != 12 {
                 return Ok(error_json(format!("aes-gcm: iv must be 12 bytes, got {}", iv.len())));
             }
             let nonce = Nonce::from_slice(&iv);

             let result = match (key.len(), op.as_str()) {
                 (16, "encrypt") => {
                     let cipher = match Aes128Gcm::new_from_slice(&key) {
                         Ok(c) => c, Err(e) => return Ok(error_json(e.to_string())),
                     };
                     match cipher.encrypt(nonce, data.as_slice()) {
                         Ok(ct) => hex::encode(ct), Err(e) => return Ok(error_json(e.to_string())),
                     }
                 }
                 (16, "decrypt") => {
                     let cipher = match Aes128Gcm::new_from_slice(&key) {
                         Ok(c) => c, Err(e) => return Ok(error_json(e.to_string())),
                     };
                     match cipher.decrypt(nonce, data.as_slice()) {
                         Ok(pt) => hex::encode(pt), Err(e) => return Ok(error_json(format!("aes-gcm: decrypt/auth failed: {}", e))),
                     }
                 }
                 (32, "encrypt") => {
                     let cipher = match Aes256Gcm::new_from_slice(&key) {
                         Ok(c) => c, Err(e) => return Ok(error_json(e.to_string())),
                     };
                     match cipher.encrypt(nonce, data.as_slice()) {
                         Ok(ct) => hex::encode(ct), Err(e) => return Ok(error_json(e.to_string())),
                     }
                 }
                 (32, "decrypt") => {
                     let cipher = match Aes256Gcm::new_from_slice(&key) {
                         Ok(c) => c, Err(e) => return Ok(error_json(e.to_string())),
                     };
                     match cipher.decrypt(nonce, data.as_slice()) {
                         Ok(pt) => hex::encode(pt), Err(e) => return Ok(error_json(format!("aes-gcm: decrypt/auth failed: {}", e))),
                     }
                 }
                 (kl, _) => return Ok(error_json(format!("aes-gcm: unsupported key length {}b", kl * 8))),
             };
             return Ok(result);
         }

         let iv = match iv_hex.as_deref().map(hex::decode) {
             Some(Err(e)) => return Ok(error_json(format!("aes: bad iv hex: {}", e))),
             Some(Ok(v)) => v,
             None => vec![0u8; 16],
         };

         let result = match (key.len(), mode.as_str(), op.as_str()) {
             (16, "cbc", "encrypt") => {
                 let enc = match Encryptor::<Aes128>::new_from_slices(&key, &iv) {
                     Ok(e) => e, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 hex::encode(enc.encrypt_padded_vec_mut::<Pkcs7>(&data))
             }
             (16, "cbc", "decrypt") => {
                 let dec = match Decryptor::<Aes128>::new_from_slices(&key, &iv) {
                     Ok(d) => d, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 match dec.decrypt_padded_vec_mut::<Pkcs7>(&data) {
                     Ok(d) => hex::encode(d), Err(e) => return Ok(error_json(e.to_string())),
                 }
             }
             (32, "cbc", "encrypt") => {
                 let enc = match Encryptor::<Aes256>::new_from_slices(&key, &iv) {
                     Ok(e) => e, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 hex::encode(enc.encrypt_padded_vec_mut::<Pkcs7>(&data))
             }
             (32, "cbc", "decrypt") => {
                 let dec = match Decryptor::<Aes256>::new_from_slices(&key, &iv) {
                     Ok(d) => d, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 match dec.decrypt_padded_vec_mut::<Pkcs7>(&data) {
                     Ok(d) => hex::encode(d), Err(e) => return Ok(error_json(e.to_string())),
                 }
             }
             (kl, _, _) => return Ok(error_json(format!("unsupported key length {}b or mode {}", kl * 8, mode))),
         };
         Ok(result)
     },
    )?)?;

    Ok(())
}