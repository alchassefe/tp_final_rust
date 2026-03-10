use crate::model::{Command, Response};
use crate::store::{Entry, Store};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub async fn process_command(cmd: Command, store: &Store) -> Response {
    match cmd {
        Command::Ping => Response::ok(),

        Command::Set { key, value } => {
            store.lock().unwrap().insert(
                key,
                Entry {
                    value,
                    expires_at: None,
                },
            );
            Response::ok()
        }

        Command::Get { key } => {
            let mut store_data = store.lock().unwrap();

            if let Some(entry) = store_data.get(&key) {
                if entry.is_expired() {
                    store_data.remove(&key);
                    return Response {
                        value: Some(serde_json::Value::Null),
                        ..Response::ok()
                    };
                }
                return Response {
                    value: Some(serde_json::Value::String(entry.value.clone())),
                    ..Response::ok()
                };
            }
            Response {
                value: Some(serde_json::Value::Null),
                ..Response::ok()
            }
        }

        Command::Del { key } => {
            let count = if store.lock().unwrap().remove(&key).is_some() {
                1
            } else {
                0
            };
            Response {
                count: Some(count),
                ..Response::ok()
            }
        }

        Command::Keys => {
            let mut store_data = store.lock().unwrap();

            // Nettoyage de toute les clés expirées
            store_data.retain(|_, entry| !entry.is_expired());

            let keys = store_data.keys().cloned().collect();
            Response {
                keys: Some(keys),
                ..Response::ok()
            }
        }

        Command::Expire { key, seconds } => {
            if let Some(entry) = store.lock().unwrap().get_mut(&key) {
                entry.expires_at = Some(Instant::now() + Duration::from_secs(seconds));
            }
            Response::ok()
        }

        Command::Ttl { key } => {
            let mut store_data = store.lock().unwrap();
            let ttl = match store_data.get(&key) {
                None => -2,
                Some(entry) => {
                    if entry.is_expired() {
                        store_data.remove(&key);
                        -2
                    } else {
                        entry
                            .expires_at
                            .map_or(-1, |at| (at - Instant::now()).as_secs() as i64)
                    }
                }
            };
            Response {
                ttl: Some(ttl),
                ..Response::ok()
            }
        }

        Command::Incr { key } => incr_decr_key(store, key, 1),
        Command::Decr { key } => incr_decr_key(store, key, -1),

        Command::Save => {
            let json = {
                let store_data = store.lock().unwrap();
                let data: HashMap<_, _> = store_data
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.value.as_str()))
                    .collect();
                serde_json::to_string(&data).unwrap()
            };

            if tokio::fs::write("dump.json", json).await.is_err() {
                return Response::error("file error");
            }
            Response::ok()
        }

        Command::Unknown => Response::error("unknown command"),
    }
}

// Méhode pour réutiliser la même logique entre incr et decr
fn incr_decr_key(store: &Store, key: String, delta: i64) -> Response {
    let mut store_data = store.lock().unwrap();
    let entry = store_data.entry(key).or_insert_with(|| Entry {
        value: "0".to_string(),
        expires_at: None,
    });

    let val = match entry.value.parse::<i64>() {
        Ok(v) => v,
        Err(_) => return Response::error("not an integer"),
    };

    let new_val = val + delta;
    entry.value = new_val.to_string();

    Response {
        value: Some(serde_json::Value::Number(new_val.into())),
        ..Response::ok()
    }
}
