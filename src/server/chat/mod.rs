use actix_web::{HttpRequest, web, HttpResponse, Error, get};
use actix_ws::{Message as WsMessage, Session};
use super::AppStateData;
use std::sync::Arc;

pub struct Peer {
    pub session: Session
}

impl Peer {
    pub async fn receive_message(&mut self, message: &Message){
        self.session.text(message.content.clone()).await.unwrap();
        println!("loli");
    }
}

#[derive(Clone, Debug)]
pub struct Message {
    content: String
}

#[derive(Default)]
pub struct State {
    peers: Vec<Arc<tokio::sync::Mutex<Peer>>>,
    messages: Vec<Message>
}

impl State {
    pub fn connect(&mut self, peer: &Arc<tokio::sync::Mutex<Peer>>){
        self.peers.push(peer.clone());
    }
  
    pub async fn send_message(&mut self, message: &Message){
        self.messages.push(message.clone());
        println!("messages: {:?}", self.messages);
        for peer in &mut self.peers {
            println!("peer sent");
            peer.lock().await.receive_message(message).await;
        }
    }
}

#[get("/chat")]
pub async fn service(app_state: AppStateData, request: HttpRequest, body: web::Payload) -> Result<HttpResponse, Error> {
    let (response, mut session, mut message_stream) = actix_ws::handle(&request, body)?;

    let peer = Arc::new(tokio::sync::Mutex::new(Peer {
        session: session.clone()
    }));

    app_state.chat_state.lock().unwrap().connect(&peer);
    let chat_state = app_state.chat_state.clone();

    actix_rt::spawn(async move {
        while let Some(Ok(message)) = message_stream.recv().await {
            match message {
                WsMessage::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                WsMessage::Text(message) => {
                    println!("Got text, {}", &message);
                    let message = Message {
                        content: message.to_string()
                    };
                    chat_state.lock().unwrap().send_message(&message).await;
                },
                _ => break
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
 // that was honestly a bit hard, but I did it!!!!! lol