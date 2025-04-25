use std::net::{SocketAddr, IpAddr};
use std::path::Path;
use std::fmt::Debug;
use tokio::{io, fs};
use tracing::info;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
  pub dns_listen: SocketAddr,
  pub dns_zone: String,
  pub self_addr: IpAddr,
  pub http_listen: SocketAddr,
  pub db_url: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dns_listen: ([0, 0, 0, 0], 5353).into(),
      dns_zone: "example.com.".to_string(),
      self_addr: [127, 0, 0, 1].into(),
      http_listen: ([0, 0, 0, 0], 3000).into(),
      db_url: "postgresql://localhost/floppa-dns".to_string(),
    }
  }
}

impl Config {
  pub async fn load<P: AsRef<Path> + Debug>(file: P) -> Result<Self, io::Error> {
    let config = match fs::read_to_string(&file).await {
      Ok(contents) => toml::from_str(&contents).map_err(io::Error::other)?,
      Err(err) => match err.kind() {
        io::ErrorKind::NotFound => {
          let default_config = Config::default();
          info!("creating default config {:?}", file);
          fs::write(
            &file,
            toml::to_string_pretty(&default_config).map_err(io::Error::other)?,
          )
          .await?;
          default_config
        }
        _ => return Err(err),
      },
    };
    Ok(config)
  }
}
