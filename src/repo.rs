use std::sync::Arc;

use axum::async_trait;
use redis::Client;
use serde::{Deserialize, Serialize};

pub type Session = Vec<Peer>;

#[derive(Serialize, Deserialize)]
pub struct Offer {
    #[serde(rename = "type")]
    offer_type: String,
    sdp: String,
}
#[derive(Serialize, Deserialize)]
pub struct Peer {
    peer_id: String,
    offer: Offer,
}

impl RedisSessionRepo {
    pub fn new(client: Client) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

pub type DynSessionRepo = Arc<dyn SessionRepo + Send + Sync>;


#[async_trait]
pub trait SessionRepo {
    async fn find(&self, session_id: String) -> Result<Session, SessionRepoError>;
    async fn create(&self, session: Session) -> Result<bool, SessionRepoError>;
}



#[derive(Clone)]
pub struct RedisSessionRepo {
    client: Arc<Client>,
}

#[async_trait]
impl SessionRepo for RedisSessionRepo {
   

    async fn find(&self, session_id: String) -> Result<Session, SessionRepoError> {
        unimplemented!()
    }

    async fn create(&self, session: Session) -> Result<bool, SessionRepoError> {
        unimplemented!()
    }
}


#[derive(Debug)]
enum SessionRepoError {
    #[allow(dead_code)]
    NotFound,
}
