use mysql::{
    Pool,
    prelude::*,
};
use std::{
    error::Error,
    fmt,
    collections::HashMap,
};

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(mysql::Error),
    QueryError(mysql::Error)
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            DatabaseError::ConnectionError(error) => format!("Couldn\'t connect to the database. Error: {}", error),
            DatabaseError::QueryError(error) => format!("Couldn\'t execute query. Error: {}", error)
        };

        write!(f, "{}", message)
    }
}

impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DatabaseError::ConnectionError(e) => e.source(),
            DatabaseError::QueryError(e) => e.source()
        }
    }
}

pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub db: String,
    pub host: String,
    pub port: u16,
}

impl From<&HashMap<String, String>> for DatabaseConfig {
    fn from(hashmap: &HashMap<String, String>) -> DatabaseConfig {
        let host = hashmap.get("database_host")
            .expect("database_host not defined in Settings.toml!")
            .to_owned();
        let db = hashmap.get("database_db")
            .expect("database_db not defined in Settings.toml!")
            .to_owned();
        let user = hashmap.get("database_user")
            .expect("database_user not defined in Settings.toml!")
            .to_owned();
        let password = hashmap.get("database_password")
            .expect("database_password not defined in Settings.toml!")
            .to_owned();
        let port: u16 = hashmap.get("database_port")
            .expect("database_port not defined in Settings.toml!")
            .to_owned()
            .parse()
            .expect("database_port is not valid!");
            

        DatabaseConfig {
            user,
            password,
            db,
            host,
            port,
        }
    }
}

pub struct Database {
    connection: Pool
}

impl Database {

    pub fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let db_url = format!("mysql://{user}:{password}@{host}:{port}/{db}", 
            user = config.user,
            password = config.password,
            host = config.host,
            port = 3306,
            db = config.db);

        match Pool::new(db_url) {
            Ok(connection) => Ok(Database { connection }),
            Err(e) => Err(DatabaseError::ConnectionError(e))
        }
    }
    
    pub fn query(&mut self, query: &str) -> Result<(), DatabaseError> {
        match self.connection.get_conn() {
            Ok(mut connection) => match connection.as_mut().query_drop(query) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(DatabaseError::QueryError(e))
                },
            Err(e) => Err(DatabaseError::ConnectionError(e))
        }
    }
}
