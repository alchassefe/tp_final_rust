#[cfg(test)]
mod tests {
    use mini_redis::model::{Command};
    use mini_redis::command::process_command;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_set_and_get() {
        let store = Arc::new(Mutex::new(HashMap::new()));
        
        let set_cmd = Command::Set { 
            key: "rust".into(), 
            value: "test".into() 
        };
        process_command(set_cmd, &store).await;

        let get_cmd = Command::Get { key: "rust".into() };
        let resp = process_command(get_cmd, &store).await;

        assert_eq!(resp.status, "ok");
        assert_eq!(resp.value, Some(serde_json::Value::String("test".into())));
    }

    #[tokio::test]
    async fn test_incr_new_key() {
        let store = Arc::new(Mutex::new(HashMap::new()));
        
        let cmd = Command::Incr { key: "counter".into() };
        let resp = process_command(cmd, &store).await;

        assert_eq!(resp.status, "ok");
        if let Some(serde_json::Value::Number(n)) = resp.value {
            assert_eq!(n.as_i64(), Some(1));
        } else {
            panic!("Devrait retourner le nombre 1");
        }
    }
}