use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

pub struct Client {
    pub nickname: Mutex<Option<String>>,
    pub username: Mutex<Option<String>>,
    pub writer: Mutex<OwnedWriteHalf>
}

impl Client {
    pub async fn display_name(&self) -> String {
        let nick = self.nickname.lock().await;
        let user = self.username.lock().await;
        match (&*nick, &*user) {
            (Some(nick), Some(user)) => format!("{}!{}", nick, user),
            (Some(nick), None) => nick.clone(),
            (None, Some(user)) => user.clone(),
            (None, None) => "Anonymous".to_string()
        }
    }
}