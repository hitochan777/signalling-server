use anyhow::{bail, Result};
use std::collections::HashMap;
use tokio::sync::mpsc as tokio_mpsc;

pub struct PubSub<T> {
    conn_map: HashMap<String, tokio_mpsc::UnboundedSender<T>>,
}

impl<T: Send + Sync + 'static> PubSub<T> {
    pub fn new() -> Self {
        Self {
            conn_map: HashMap::new(),
        }
    }
    pub fn publish(&self, target: &str, data: T) -> Result<()> {
        if let Some(tx) = self.conn_map.get(target) {
            tx.send(data)?;
            Ok(())
        } else {
            bail!("target {} not found", target);
        }
    }
    pub fn add_subscriber(
        &mut self,
        target: &str,
    ) -> Result<tokio_mpsc::UnboundedReceiver<T>, String> {
        let (tx, rx) = tokio_mpsc::unbounded_channel::<T>();
        // insert or update the tx
        self.conn_map.insert(target.to_string(), tx);
        Ok(rx)
    }
    pub fn remove_subscriber(&mut self, target: &str) {
        self.conn_map.remove(target);
    }
}
