use std::{sync::{Arc, Mutex}};

type PeerId = String;

#[derive(Clone)]
struct PeerInfo {
    peer_id: PeerId,
    updated_at: u64,
}

struct PeerStatusService {
    peer_info_list: Vec<PeerInfo>,
}

impl PeerStatusService {
    fn fetch(&self) -> Vec<PeerInfo> {
        return vec![];
    }
    fn update(&mut self, peer_info_list: Vec<PeerInfo>) -> Result<(), ()> {
        self.peer_info_list = peer_info_list;
    }
}

struct LeaderRepository {

}

struct LeaderSelector {
    peer_status_service: PeerStatusService,
    leader_repository: LeaderRepository,
    selected_peer_id: Option<PeerId>,
}

impl LeaderSelector {
    fn new() -> Self {

    }

    fn handle_connect(&self, peer_id: PeerId, info: PeerInfo) -> Result<(), ()> {
        let peer = self.peer_status_service.fetch_by_id(user_id, peerId)?;
        let new_peer = PeerInfo {
            ...peer,
            ...peer_info
            updated_at: now,
        }
        self.peer_status_service.save(user_id, peer_id);
        self.select_leader();
    }

    fn handle_disconnect(&self) -> Result<PeerId, ()> {
        let peer = self.peer_status_service.delete(user_id, peerId)?;
        if leader != self.leader_repository.fetch(user_id) {
            self.leader_repository.save(user_id, leader);
            self.broadcast(user_id, leader);
        }
    }

    fn select_leader(&self) {
        let peers = self.peer_status_service.fetch_all(user_id)?;
        let leader = self.select(peers)?;
        if leader != self.leader_repository.fetch(user_id) {
            self.leader_repository.save(user_id, leader);
            self.broadcast(user_id, leader);
        }
    }

    fn select(&self, peers: Vec<PeerInfo>) -> Result<PeerId, ()> {
        if peers.is_empty() {
            return Err(());
        }
        // for now choose the first one
        Ok(peers[0].peer_id)
    }

    /*fn gc(&self) -> () {
        let info_list = self.peer_status_service.fetch();
        let new_list = info_list.iter().filter(|&peer_info| now - peer_info.updated_at > 60).map(|peer_info| peer_info.clone()).collect();
        let 
        self.peer_status_service.update(new_list);
    }*/
}

fn main() {

}

// watcher ---------------> peer_status_service
//    | (disconnect)          
//    v
// leader_selector -------> peer_status_service 
//    ^ 
//    | (connect, disconnect, ka)
// API (SSE)      