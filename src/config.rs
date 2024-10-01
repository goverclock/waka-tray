use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub api_key: String,
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

    // if the config file doesn't exist, create it
    let mut conf = read_config(&path);
    if conf.is_err() {
        write_config(&Config::default(), &path).unwrap();
        conf = read_config(&path);
    }

    log::info!("read config result={:#?}", conf);
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
