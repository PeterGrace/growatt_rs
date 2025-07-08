use anyhow::bail;
use crate::mqtt_actor::{MqttActor, MqttMessage, run_mqtt_actor};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct MqttActorHandler {
    pub sender: mpsc::Sender<MqttMessage>,
}

impl MqttActorHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = MqttActor::new(receiver);
        tokio::spawn(run_mqtt_actor(actor));
        Self { sender }
    }
    pub async fn publish(&self, topic: String, payload: String) -> anyhow::Result<()> {
        let (send, recv) = oneshot::channel();
        let msg = MqttMessage::Publish {
            topic,
            payload,
            respond_to: send,
        };

        if let Err(e) = self.sender.send(msg).await {
            let msg = format!("Unable to send message: {e}");
            error!("{msg}");
            bail!(msg);
        } else {
            if let Err(e) = recv.await {
                let msg = format!("Actor task died?: {e}");
                error!("Actor task died?: {e}");
                bail!(msg);
            }
        }
        Ok(())
    }
}
