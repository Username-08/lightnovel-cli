mod screen;

use screen::Screen;
use std::env;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create config directory if it doesnt exist
    let path = create_config_dir();

    Screen::new(path).await?;

    Ok(())
}

pub fn create_config_dir() -> String {
    // create config dir if doesnt exist already
    // fs::create_dir_all("/some/dir")?;
    let mut path: String;
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            path = v.to_string();
            path.push_str("/lightnovel-cli");
            fs::create_dir_all(path.as_str()).unwrap();
            path.push_str("/novels.txt");
            if Path::new(path.as_str()).exists() {
                ()
            } else {
                fs::File::create(&path).unwrap();
            }
        }
        Err(_) => {
            path = env::var("HOME").unwrap().to_string();
            path.push_str("/.config/lightnovel-cli");
            fs::create_dir_all(path.as_str()).unwrap();
            path.push_str("/novels.txt");
            if Path::new(path.as_str()).exists() {
                ()
            } else {
                fs::File::create(&path).unwrap();
            }
        }
    };

    path
}
