mod database;
mod clima;
mod mqtt;

extern crate config;

use crate::database::*;
use crate::mqtt::*;
use crate::clima::*;

use std::{
    thread::sleep,
    time::Duration,
    sync::{Arc, Mutex},
    collections::HashMap,
};

fn get_config() -> HashMap<String, String> {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))
        .unwrap();
    
    settings.try_into::<HashMap<String, String>>().unwrap()
}


fn get_database(config: &HashMap<String, String>) -> Database {
    let db_config = DatabaseConfig::from(config);
    
    loop {
        match Database::new(&db_config) {
            Ok(db) => {
                println!("Connected to the database.");
                return db
            },
            Err(e) => {
                println!("Couldn\'t connect {}. \nRetrying in 5 seconds...", e);
                sleep(Duration::from_secs(5));
            }
        }
    }
}

fn get_mqtt(config: &HashMap<String, String>) -> Mqtt {
    let mqtt_config = MqttConfig::from(config);

    loop {
        match Mqtt::new(&mqtt_config) {
            Ok(mqtt) => {
                println!("Connected to mqtt server.");
                return mqtt
            },
            Err(e) => {
                println!("Couldn\'t connect to mqtt server{}. \nRetrying in 5 seconds...", e);
                sleep(Duration::from_secs(5));
            }
        }
    }
}

fn main() {
    let config = get_config();
    let mqtt = get_mqtt(&config);
    mqtt.start_listening();

    let database = get_database(&config);

    let mut clima = ClimaRepository::new(
            Arc::new(Mutex::new(database)), 
            Arc::new(Mutex::new(mqtt))
        );
    
    clima.init().unwrap();
    

    loop {

    }

}
