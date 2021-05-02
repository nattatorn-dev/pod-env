use color_eyre::Result;
use futures::prelude::*;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::core::v1::Secret;
use kube::{api::ListParams, Api, Client};
use kube_runtime::{reflector, watcher};
use std::collections::BTreeMap;
use log::{info};

#[derive(Debug)]
enum Decoded {
    Utf8(String),
    Bytes(Vec<u8>),
}

fn decode(secret: &Secret) -> BTreeMap<String, Decoded> {
    let mut res = BTreeMap::new();
    if let Some(data) = secret.data.clone() {
        for (k, v) in data {
            if let Ok(b) = std::str::from_utf8(&v.0) {
                let decoded = Decoded::Utf8(b.to_string());
                println!("{}={:?}", k, b.to_string());
                res.insert(k, decoded);
            } else {
                let decoded = Decoded::Bytes(v.0);
                println!("{}={:?}", k, decoded);
                res.insert(k, decoded);
            }
        }
    }
    res
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::var("RUST_LOG").unwrap_or("info,kube=debug".into());
    env_logger::init();
    let client = Client::try_default().await?;

    let version = client.apiserver_version().await?;
    info!("api version: {:?}", version);

    let namespace = std::env::var("NAMESPACE").unwrap_or("default".into());
    info!("namespace: {:?}", namespace);

    let api: Api<Pod> = Api::namespaced(client, &namespace);

    let store_w = reflector::store::Writer::default();
    let store = store_w.as_reader();
    let reflector = reflector(store_w, watcher(api, ListParams::default()));

    let client = Client::try_default().await?;
    let secrets: Api<Secret> = Api::namespaced(client, &namespace);

    reflector
        .try_for_each(|_event| async {
            let pods = store.state();
            println!("Current pod count: {}", pods.len());
            for pod in pods {
                println!("name: {}", pod.metadata.name.as_deref().unwrap_or(""));
                for spec in pod.spec {
                    for container in spec.containers {
                        for env in container.env {
                            for env_text in env {
                                println!("{}={}", env_text.name, env_text.value.as_deref().unwrap_or(""));
                            }
                        }
                    }
                    for volumes in spec.volumes {
                        for volume in volumes {
                            for secret in volume.secret {
                                let secret_name = secret.secret_name.as_deref().unwrap_or("");
                                let secret_resolved = secrets.get(secret_name);
                                let secrets = secret_resolved.await;
                                
                                for secret in secrets {
                                    decode(&secret);
                                }
                            }
                        }
                    }

                }
                print!("\n\n");
            }

            Ok(())
        })
        .await?;
    Ok(())
}