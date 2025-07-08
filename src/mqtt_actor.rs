use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use rumqttc::{Event, Incoming, Outgoing};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
pub struct MqttActor {
    client: AsyncClient,
    eventloop: EventLoop,

    receiver: mpsc::Receiver<MqttMessage>,
}

pub enum MqttMessage {
    Publish {
        topic: String,
        payload: String,
        respond_to: oneshot::Sender<bool>,
    },
}

impl MqttActor {
    pub fn new(receiver: mpsc::Receiver<MqttMessage>) -> Self {
        let mqttoptions = MqttOptions::new("test", "127.0.0.1", 1883);
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        MqttActor {
            client,
            eventloop,
            receiver,
        }
    }

    pub async fn handle_message(client: &AsyncClient, msg: MqttMessage) {
        let mut status: bool = true;
        match msg {
            MqttMessage::Publish {
                topic,
                payload,
                respond_to,
            } => {
                if let Err(e) =
                    client
                    .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
                    .await
                {
                    error!("Unable to publish message: {e}");
                    status = false;
                }
                let _ = respond_to.send(status);
            }
        }
    }
    fn split(self) -> (AsyncClient, EventLoop, mpsc::Receiver<MqttMessage>) {
        (self.client, self.eventloop, self.receiver)
    }
}
pub async fn run_mqtt_actor(mut actor: MqttActor) {

    let (client, mut eventloop, mut receiver) = actor.split();

    tokio::task::Builder::new()
        .name("mqtt_poll_loop")
        .spawn(async move {
            // region eventloop tending
            let mut dlq: Vec<u16> = vec![];

            loop {
                let notification = match
                    eventloop.poll().await
                {
                    Ok(event) => Some(event),
                    Err(e) => {
                        let msg = format!("Unable to poll mqtt: {e}");
                        panic!("{}", msg);
                    }
                };

                match notification {
                    Some(Event::Outgoing(o)) => {}
                    Some(Event::Incoming(i)) => {
                        match i {
                            Incoming::Disconnect => {
                                // we should do something here.
                                error!("mqtt disconnect packet received.");
                                return;
                            }
                            Incoming::ConnAck(_ca) => {
                                info!("MQTT connection established.");
                            }
                            Incoming::PubAck(pa) => {
                                dlq.retain(|x| *x != pa.pkid);
                            }
                            Incoming::PingResp => {
                                trace!("Recv MQTT PONG");
                            }
                            _ => {}
                        }
                    }
                    None => {}
                }
                //endregion
            }
        });
    loop {

        if let Ok(msg) = receiver.try_recv() {
            MqttActor::handle_message(&client, msg).await;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
