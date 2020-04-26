use crate::database::{ Database, DatabaseError };
use crate::mqtt::{ MqttError, Mqtt, Packet };

use std::{
    error::Error,
    thread,
    fmt,
};
use serde::Deserialize;
use crossbeam_channel::unbounded;
use std::sync::{ Arc, Mutex };

enum Topic {
    Weather(Packet),
}

#[derive(Debug, Default, Deserialize)]
pub struct Clima {
    id: Option<i32>,
    #[serde(alias = "hum")]
    humidity: f32,
    #[serde(alias = "temp")]
    temperature: f32
}

impl fmt::Display for Clima {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Clima {{ temperature: {}, humidity: {} }}", self.temperature, self.humidity)
    }
}

pub struct ClimaRepository {
    db: Arc<Mutex<Database>>,
    mqtt: Arc<Mutex<Mqtt>>,
    receiver: crossbeam_channel::Receiver<Topic>,
    sender: crossbeam_channel::Sender<Topic>
}

impl ClimaRepository {
    pub fn new(db: Arc<Mutex<Database>>, mqtt: Arc<Mutex<Mqtt>>) ->  ClimaRepository {
        let (sender, receiver) = unbounded();
        ClimaRepository { 
            db,
            mqtt,
            receiver, 
            sender
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_table_if_not_exist().unwrap();
        
        match self.subscribe_weather_topic() {
            Ok(()) => self.watch_mqtt_responses(),
            Err(e) => panic!("Unable to initialize. Error {}", e)
        }
    
        Ok(())
    }

    fn subscribe_weather_topic(&self) -> Result<(), MqttError> {
        let topic = "weather";
        let mut mqtt = self.mqtt.lock().unwrap();
        
        match mqtt.subscribe(topic) {
            Ok(()) => {
                let sender = self.sender.clone();

                mqtt.add_callback(topic, Box::new(move |packet: Packet|{
                    let packet = Topic::Weather(packet);
                    sender.send(packet).unwrap();
                }));
            },
            Err(e) => return Err(e)
        }

        Ok(())
    }


    fn watch_mqtt_responses (&self) {
        let db = self.db.clone();
        let receiver = self.receiver.clone();
        
        thread::spawn(move || loop {
            for message in &receiver {
                match message {
                    Topic::Weather(packet) => {
                        let clima: Clima = serde_json::from_str(&packet.payload).unwrap();
                        
                        match ClimaRepository::store(db.clone(), &clima) {
                            Ok(()) => {},
                            Err(e) => println!("[{}] Failed to store {}, cause: {}", &packet.topic, clima, e)
                        };
                    }
                }
            }
        });
    }

    fn store(db: Arc<Mutex<Database>>, clima: &Clima) -> Result<(), DatabaseError> {
        let query = format!("INSERT INTO clima (temperature, humidity)
            VALUES ({temperature:.2}, {humidity:.2})", 
            temperature = clima.temperature,
            humidity = clima.humidity
        );

        db.lock().unwrap().query(&query)
    }

    fn create_table_if_not_exist(&self) -> Result<(), DatabaseError> {
        self.db.lock().unwrap().query(r"CREATE TABLE IF NOT EXISTS clima (
            id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            date_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            temperature FLOAT(5,2),
            humidity FLOAT(5,2)
        )")
    }
}