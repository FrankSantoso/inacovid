extern crate chrono;
extern crate clap;
extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate sqlx;
extern crate tokio;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate failure;

mod endpoints;
mod helpers;
mod models;
mod queries;
mod store;

use crate::store::PgStore;
use clap::{App, Arg};
use endpoints::Request;
use failure::Error;
use std::env;
use std::fs::{create_dir_all, read_to_string};
use std::sync::Arc;

const DEFAULT_JSON_DIR: &str = "/tmp/inacovid/json_out/";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    #[serde(rename = "postgresDsn")]
    postgres_dsn: Option<String>,
    #[serde(rename = "jsonOutputDir")]
    json_dir: Option<String>,
}

impl Config {
    pub fn create_json_dir(&self) -> Result<(), std::io::Error> {
        match &self.json_dir {
            Some(path) => create_dir_all(path),
            None => create_dir_all(DEFAULT_JSON_DIR),
        }
    }
    pub fn set_db_dsn(&self) -> Result<(), Error> {
        match &self.postgres_dsn {
            Some(dsn) => {
                std::env::set_var("DATABASE_URI", std::ffi::OsStr::new(dsn));
                Ok(())
            }
            None => Err(format_err!(
                "postgres dsn is not set or its value is not valid"
            )),
        }
    }
}

fn init() -> Result<Config, Error> {
    let matches = App::new("inacovid")
        .version("0.1")
        .author("Alexander Adhyatma <alex@asiatech.dev>")
        .about("Gets Indonesian Covid19 data from gov't source, save it to postgres and json dir")
        .args(&[Arg::with_name("config")
            .help("Config file containing database dsn and json output dir")
            .long("config")
            .short('c')
            .takes_value(true)
            .required(true)])
        .get_matches();

    let c = matches.value_of("config").unwrap(); // config file is required anyway
    let path = read_to_string(c)?;
    let config_file: Config = serde_json::from_str(&path)?;
    config_file.create_json_dir()?;
    config_file.set_db_dsn()?;
    Ok(config_file)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = init()?;
    let uri = env::var("DATABASE_URL")?;
    let pool = sqlx::PgPool::new(uri.as_str())
        .await
        .map(|p| Arc::new(p))
        .expect("Could not connect to postgres");
    let store = PgStore::new(Arc::clone(&pool));
    let new_request = Request::new(
        store,
        config.json_dir.unwrap_or(DEFAULT_JSON_DIR.to_string()),
    );
    let daily = new_request.fetch_daily().await?;
    let cumulative = new_request.cumulative_stats(0).await?;
    let per_province = new_request.fetch_province().await?;
    println!("{}\n{}\n{}", daily, cumulative, per_province);
    Ok(())
}
