use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppConfig {
    pub amd64: Amd64Config,
    pub arm: ArmConfig,
}

// arm平台cgo环境
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ArmConfig {
    pub cxx_path: String,
    pub cc_path: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Amd64Config {
    pub cxx_path: String,
    pub cc_path: String,
}

impl AppConfig {
    pub fn config_file() -> PathBuf {
        let proj_dirs =
            ProjectDirs::from("com", "initcool", "xm").expect("Failed to get project directories");
        let mut config_file = PathBuf::from(proj_dirs.config_dir());
        std::fs::create_dir_all(&config_file).expect("");
        config_file.push("config.yaml");
        return config_file;
    }
    pub fn new() -> Result<AppConfig, Box<dyn Error>> {
        let mut config = AppConfig {
            amd64: Amd64Config {
                cxx_path: "".to_string(),
                cc_path: "".to_string(),
            },
            arm: ArmConfig {
                cxx_path: "".to_string(),
                cc_path: "".to_string(),
            },
        };
        match std::fs::File::open(AppConfig::config_file()) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();

                config = serde_yaml::from_str(&content)?;
            }
            Err(_e) => {
                let file = fs::File::create(AppConfig::config_file())?;
                serde_yaml::to_writer(file, &config)?;
                let config_dir = AppConfig::config_file();
                println!("{:?}", config_dir);
            }
        }
        Ok(config)
    }
    #[allow(dead_code)]
    fn create_file(&self, project_path: &Path, file_name: &str, content: &[u8]) -> io::Result<()> {
        let file_path = project_path.join(file_name);
        let mut file = File::create(&file_path)?;
        file.write_all(content)?;
        Ok(())
    }
    pub fn save(&self) {
        let file = File::options()
            .write(true)
            .open(Self::config_file())
            .expect("Failed to open config file");
        serde_yaml::to_writer(file, self).expect("Failed to serialize config");
    }
}

#[allow(dead_code)]
pub fn load_config(filename: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: AppConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}
