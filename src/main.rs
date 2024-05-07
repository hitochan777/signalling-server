use std::sync::Arc;

mod leader_selector;

fn main() {
    let leader_repository: Arc<Box<dyn leader_selector::LeaderRepository>> =
        Arc::new(Box::new(leader_selector::OnMemoryLeaderRepository::new()));
    let peer_status_repository: Arc<Box<dyn leader_selector::PeerStatusRepository>> = Arc::new(
        Box::new(leader_selector::OnMemoryPeerStatusRepository::new()),
    );
    let selector = leader_selector::LeaderSelector::new(peer_status_repository, leader_repository);
}

// watcher ---------------> peer_status_service
//    | (disconnect)
//    v
// leader_selector -------> peer_status_service
//    ^
//    | (connect, disconnect, ka)
// API (SSE)
