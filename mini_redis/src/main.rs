mod command;
mod model;
mod store;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

use crate::model::Command;
use crate::store::Store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let store: Store = Arc::new(std::sync::Mutex::new(HashMap::new()));

    // Bind un TcpListener sur 127.0.0.1:7878
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    info!("MiniRedis écoute sur 127.0.0.1:7878");

    // Accept loop : pour chaque connexion, spawn une tâche
    loop {
        let (socket, addr) = listener.accept().await?;
        let store_clone = Arc::clone(&store);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, store_clone).await {
                error!("Erreur client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream, store: Store) -> tokio::io::Result<()> {
    let (reader, mut writer) = socket.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    // Dans chaque tâche : lire les requêtes JSON ligne par ligne,
    //    traiter la commande, envoyer la réponse JSON + '\n'
    loop {
        line.clear();
        if buf_reader.read_line(&mut line).await? == 0 {
            break;
        }

        let response = match serde_json::from_str::<Command>(&line) {
            Ok(cmd) => command::process_command(cmd, &store).await,
            Err(_) => model::Response::error("invalid json"),
        };

        let mut json_resp = serde_json::to_string(&response).unwrap();
        json_resp.push('\n');
        writer.write_all(json_resp.as_bytes()).await?;
    }
    Ok(())
}
