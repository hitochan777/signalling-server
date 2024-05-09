use anyhow::{anyhow, bail, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

type UserId = String;
type PeerId = String;

#[derive(Clone)]
pub struct PeerInfo {
    peer_id: PeerId,
    updated_at: u64,
}

#[async_trait::async_trait]
pub trait PeerStatusRepository: Send + Sync {
    async fn fetch(&self, user_id: UserId, peer_id: PeerId) -> Result<PeerInfo>;
    async fn fetch_all(&self, user_id: UserId) -> Result<Vec<PeerInfo>>;
    async fn delete(&self, user_id: UserId, peer_id: PeerId) -> Result<()>;
    async fn update(&self, user_id: UserId, peer_info: PeerInfo) -> Result<()>;
}

pub struct OnMemoryPeerStatusRepository {
    peer_info_map: Mutex<HashMap<UserId, HashMap<PeerId, PeerInfo>>>,
}

impl OnMemoryPeerStatusRepository {
    pub fn new() -> Self {
        Self {
            peer_info_map: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PeerStatusRepository for OnMemoryPeerStatusRepository {
    async fn fetch(&self, user_id: UserId, peer_id: PeerId) -> Result<PeerInfo> {
        return self
            .peer_info_map
            .lock()
            .unwrap()
            .get(&user_id)
            .and_then(|v| v.get(&peer_id))
            .and_then(|info| Some(info.clone()))
            .ok_or(anyhow!("not found"));
    }
    async fn fetch_all(&self, user_id: UserId) -> Result<Vec<PeerInfo>> {
        return self
            .peer_info_map
            .lock()
            .unwrap()
            .get(&user_id)
            .and_then(|info| Some(info.values().map(|v| v.clone()).collect()))
            .or_else(|| Some(vec![]))
            .ok_or(anyhow!("unknown error"));
    }
    async fn delete(&self, user_id: UserId, peer_id: PeerId) -> Result<()> {
        if let Some(info) = self.peer_info_map.lock().unwrap().get_mut(&user_id) {
            info.remove(&peer_id);
        } else {
            bail!(format!("user_id: {} not found", user_id));
        }
        Ok(())
    }
    async fn update(&self, user_id: UserId, peer_info: PeerInfo) -> Result<()> {
        let mut map_lock = self.peer_info_map.lock().unwrap();
        if !map_lock.contains_key(&user_id) {
            map_lock.insert(user_id.clone(), HashMap::<PeerId, PeerInfo>::new());
        }
        let user_map = map_lock.get_mut(&user_id).unwrap();
        if let Some(info) = user_map.get_mut(&peer_info.peer_id) {
            *info = peer_info;
        } else {
            user_map.insert(peer_info.peer_id.clone(), peer_info);
        }
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait LeaderRepository: Send + Sync {
    async fn fetch(&self, user_id: UserId) -> Result<PeerId>;
    async fn update(&self, user_id: UserId, leader_id: PeerId) -> Result<()>;
}

pub struct OnMemoryLeaderRepository {
    leader_map: Mutex<HashMap<UserId, PeerId>>,
}

impl OnMemoryLeaderRepository {
    pub fn new() -> Self {
        Self {
            leader_map: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl LeaderRepository for OnMemoryLeaderRepository {
    async fn fetch(&self, user_id: UserId) -> Result<PeerId> {
        return self
            .leader_map
            .lock()
            .unwrap()
            .get(&user_id)
            .and_then(|info| Some(info.clone()))
            .ok_or(anyhow!("not found"));
    }

    async fn update(&self, user_id: UserId, new_leader_id: PeerId) -> Result<()> {
        let mut map = self.leader_map.lock().unwrap();
        if let Some(leader) = map.get_mut(&user_id) {
            *leader = new_leader_id;
        } else {
            map.insert(user_id.clone(), new_leader_id.clone());
        };
        Ok(())
    }
}

#[derive(Clone)]
pub struct LeaderSelector {
    peer_status_repository: Arc<Box<dyn PeerStatusRepository>>,
    leader_repository: Arc<Box<dyn LeaderRepository>>,
}

impl LeaderSelector {
    pub fn new(
        peer_status_repository: Arc<Box<dyn PeerStatusRepository>>,
        leader_repository: Arc<Box<dyn LeaderRepository>>,
    ) -> Self {
        Self {
            peer_status_repository,
            leader_repository,
        }
    }

    pub async fn handle_connect(&self, user_id: UserId, info: PeerInfo) -> Result<()> {
        let peer = match self
            .peer_status_repository
            .fetch(user_id.clone(), info.peer_id.clone())
            .await
        {
            Ok(peer) => peer,
            Err(_) => PeerInfo {
                peer_id: info.peer_id.clone(),
                // TODO: set updated_at to now
                updated_at: 0,
            },
        };
        let updated_info = PeerInfo {
            updated_at: info.updated_at,
            ..peer
        };
        self.peer_status_repository
            .update(user_id.clone(), updated_info)
            .await?;
        self.select_leader(user_id.clone()).await?;
        Ok(())
    }

    pub async fn handle_disconnect(&self, user_id: UserId, peer_id: PeerId) -> Result<()> {
        self.peer_status_repository
            .delete(user_id.clone(), peer_id)
            .await?;
        self.select_leader(user_id.clone()).await?;
        Ok(())
    }

    async fn select_leader(&self, user_id: UserId) -> Result<PeerId> {
        let peers = self
            .peer_status_repository
            .fetch_all(user_id.clone())
            .await?;
        let leader = self.select(peers)?;
        let current_leader = self.leader_repository.fetch(user_id.clone()).await;
        if current_leader.is_err() || leader != current_leader.unwrap() {
            self.leader_repository
                .update(user_id.clone(), leader.clone())
                .await?;
            // self.broadcast(user_id, leader);
        }
        Ok(leader)
    }

    fn select(&self, peers: Vec<PeerInfo>) -> Result<PeerId> {
        if peers.is_empty() {
            bail!("Cannot find any peers");
        }
        // for now choose the first one
        Ok(peers[0].peer_id.clone())
    }

    /*fn gc(&self) -> () {
        let info_list = self.peer_status_service.fetch();
        let new_list = info_list.iter().filter(|&peer_info| now - peer_info.updated_at > 60).map(|peer_info| peer_info.clone()).collect();
        let
        self.peer_status_service.update(new_list);
    }*/
}

#[cfg(test)]
mod test {
    use core::panic;

    use super::*;

    fn create_selector() -> LeaderSelector {
        let leader_repository: Arc<Box<dyn LeaderRepository>> =
            Arc::new(Box::new(OnMemoryLeaderRepository::new()));
        let peer_status_repository: Arc<Box<dyn PeerStatusRepository>> =
            Arc::new(Box::new(OnMemoryPeerStatusRepository::new()));
        let selector = LeaderSelector::new(peer_status_repository, leader_repository);
        return selector;
    }

    #[tokio::test]
    async fn test_leader_selection_returns_error_when_no_peer() -> () {
        let selector = create_selector();
        let leader = selector.select_leader(String::from("user1")).await;
        assert!(leader.is_err())
    }

    #[tokio::test]
    async fn test_leader_selection_returns_first_peer_when_multiple_available() {
        let selector = create_selector();
        selector
            .handle_connect(
                String::from("user1"),
                PeerInfo {
                    peer_id: String::from("peer1"),
                    updated_at: 1,
                },
            )
            .await
            .unwrap();
        selector
            .handle_connect(
                String::from("user1"),
                PeerInfo {
                    peer_id: String::from("peer2"),
                    updated_at: 1,
                },
            )
            .await
            .unwrap();
        let leader = selector.select_leader(String::from("user1")).await.unwrap();
        assert_eq!(leader, String::from("peer1"))
    }

    #[tokio::test]
    async fn test_leader_selection_returns_second_peer_when_first_peer_is_disconnected() {
        let selector = create_selector();
        selector
            .handle_connect(
                String::from("user1"),
                PeerInfo {
                    peer_id: String::from("peer1"),
                    updated_at: 1,
                },
            )
            .await
            .unwrap();
        selector
            .handle_connect(
                String::from("user1"),
                PeerInfo {
                    peer_id: String::from("peer2"),
                    updated_at: 1,
                },
            )
            .await
            .unwrap();
        selector
            .handle_disconnect(String::from("user1"), String::from("peer1"))
            .await
            .unwrap();
        let leader = selector.select_leader(String::from("user1")).await.unwrap();
        assert_eq!(leader, String::from("peer2"))
    }
}
