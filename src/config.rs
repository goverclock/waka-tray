use serde::{Deserialize, Serialize};
use std::fs;
use toml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
        }
    }
}

const CONFIG_FILE_NAME: &str = "waka-tray.toml";

pub fn config_path() -> String {
    let mut config_path = dirs::home_dir().unwrap();
    config_path.push(".config");
    config_path.push(CONFIG_FILE_NAME);
    config_path.to_str().unwrap().to_owned()
}

pub fn init_config() -> Config {
    let path = config_path();
    let _ = fs::File::create_new(&path);

    let mut conf = read_config(&path);
    if conf.is_err() {
        write_config(&Config::default(), &path).unwrap();
        conf = read_config(&path);
    }
    println!("read config={:#?}", conf);
    conf.unwrap()
}

fn read_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let text = String::from_utf8(data)?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}

fn write_config(config: &Config, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = toml::to_string(config)?;
    std::fs::write(path, text)?;
    Ok(())
}
