use rumqtt::{ 
    MqttClient, 
    MqttOptions, 
    QoS, 
    Notification
};
extern crate crossbeam_channel;

use crossbeam_channel::Receiver;
use std::{
    collections::HashMap, 
    sync::{Arc, Mutex},
    thread,
    fmt
};

pub struct Packet {
    pub payload: String,
    pub topic: String,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}", self.topic, self.payload)
    }
}

#[derive(Debug)]
pub enum MqttError {
    ConnectionError(rumqtt::ConnectError),
    SubscriptionError(Box<rumqtt::ClientError>)
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            MqttError::ConnectionError(error) => format!("Couldn\'t connect to the Mqtt. Error: {}", error),
            MqttError::SubscriptionError(error) => format!("Couldn\'t subscribe to topic. Error: {}", error)
        };

        write!(f, "{}", message)
        
    }
}

pub struct MqttConfig {
    pub host: String,
    pub id: String,
    pub port: u16,
}

impl From<&HashMap<String, String>> for MqttConfig {
    fn from(hashmap: &HashMap<String, String>) -> MqttConfig {
        let host = hashmap.get("mqtt_host")
            .expect("mqtt_host not defined in Settings.toml!")
            .to_owned();
        let id = hashmap.get("mqtt_id")
                .expect("mqtt_id not defined in Settings.toml!")
                .to_owned();
        let port: u16 = hashmap.get("mqtt_port")
                .expect("mqtt_port not defined in Settings.toml!")
                .to_owned()
                .parse()
                .expect("mqtt_port is not valid!");
                
        
        MqttConfig {
            host,
            id,
            port,
        }
    }
}

type CallbackHashMap = Arc<Mutex<HashMap<String, Box<dyn Fn(Packet) + Send>>>>;
pub struct Mqtt {
    notifications: Receiver<Notification>,
    client: MqttClient,
    callbacks: CallbackHashMap,
}

impl Mqtt {

    pub fn new(config: &MqttConfig) -> Result<Self,  MqttError> {
        let mqtt_options = MqttOptions::new(config.id.clone(), config.host.clone(), config.port);

        match MqttClient::start(mqtt_options) {
            Ok((client, notifications)) => Ok(Mqtt {
                    client,
                    notifications,
                    callbacks: Arc::new(Mutex::new(HashMap::new()))
                }),
            Err(e) => Err(MqttError::ConnectionError(e))
        }
    }

    pub fn subscribe(&mut self, topic: &str) -> Result<(), MqttError> {
        match self.client.subscribe(topic, QoS::AtLeastOnce) {
            Err(e) => Err(MqttError::SubscriptionError(Box::new(e))),
            Ok(_) => Ok(())
        }
    }
    
    pub fn add_callback(&mut self, topic: &str, callback: Box<dyn Fn(Packet) + Send>) {
        self.callbacks
            .lock()
            .unwrap()
            .insert(topic.to_string(), callback);
    }

    pub fn start_listening(&self) {
        let notifications = self.notifications.clone();
        let callbacks = Arc::clone(&self.callbacks);

        thread::spawn(move | | loop {
            for notification in &notifications {
                if let Notification::Publish(packet) = notification {
                    if let Some(ref callback) = callbacks.lock().unwrap().get(&packet.topic_name) {
                        let pack = Packet {
                            payload: String::from_utf8(packet.payload.to_vec()).unwrap(),
                            topic: packet.topic_name
                        };
                        
                        println!("{}", pack);

                        callback(pack);
                    }
                }

            }
        });
    }

}