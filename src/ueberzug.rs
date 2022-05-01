//! # Ueberzug-rs
//! [Ueberzug-rs](https://github.com/Adit-Chauhan/Ueberzug-rs) This project provides simple bindings to that [ueberzug](https://github.com/seebye/ueberzug) to draw images in the terminal.
//!
//!This code was inspired from the [termusic](https://github.com/tramhao/termusic) to convert their specilized approach to a more general one.
//!
//! ## Examples
//! this example will draw image for 2 seconds, erase the image and wait 1 second before exiting the program.
//! ```
//! use std::thread::sleep;
//! use std::time::Duration;
//! use ueberzug::{UeConf,Scalers};
//!
//! let a = ueberzug::Ueberzug::new();
//! // Draw image
//! // See UeConf for more details
//! a.draw(&UeConf {
//!     identifier: "crab",
//!     path: "ferris.png",
//!     x: 10,
//!     y: 2,
//!     width: Some(10),
//!     height: Some(10),
//!     scaler: Some(Scalers::FitContain),
//!     ..Default::default()
//! });
//! sleep(Duration::from_secs(2));
//! // Only identifier needed to clear image
//! a.clear("crab");
//! sleep(Duration::from_secs(1));
//! ```

use std::fmt;
use std::io::Write;
use std::process::Child;
use std::process::Stdio;
use std::sync::RwLock;

/// Main Ueberzug Struct
pub struct Ueberzug(RwLock<Option<Child>>);

impl Ueberzug {
    /// Creates the Default Ueberzug instance
    /// One instance can handel multiple images provided they have different identifiers
    pub fn new() -> Self {
        Self(RwLock::new(None))
    }
    /// Draws the Image using UeConf
    pub fn draw(&self, config: &UeConf) {
        let cmd = config.to_json();
        if let Err(e) = self.run(&cmd) {
            println!(
                "could not draw {},from path {}",
                config.identifier, config.path
            );
            println!("{}", e);
        };
    }
    /// Clear the drawn image only requires the identifier
    pub fn clear(&self, identifier: &str) {
        let config = UeConf {
            action: Actions::Remove,
            identifier: identifier.to_string(),
            ..Default::default()
        };
        let cmd = config.to_json();
        self.run(&cmd).expect("Failed to Clear Image");
    }

    fn run(&self, cmd: &str) -> Result<(), std::io::Error> {
        let mut ueberzug = self.0.write().unwrap();
        if ueberzug.is_none() {
            *ueberzug = Some(
                std::process::Command::new("ueberzug")
                    .args(&["layer", "--silent"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?,
            );
        }

        let stdin = (*ueberzug).as_mut().unwrap().stdin.as_mut().unwrap();
        stdin.write_all(cmd.as_bytes())?;

        Ok(())
    }
}

/// Action enum for the json value
pub enum Actions {
    Add,
    Remove,
}

impl fmt::Display for Actions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Actions::Add => write!(f, "add"),
            &Actions::Remove => write!(f, "remove"),
        }
    }
}
/// Scalers that can be applied to the image and are supported by ueberzug
#[derive(Clone, Copy)]
pub enum Scalers {
    Crop,
    Distort,
    FitContain,
    Contain,
    ForcedCover,
    Cover,
}

impl fmt::Display for Scalers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Scalers::Contain => write!(f, "contain"),
            &Scalers::Cover => write!(f, "cover"),
            &Scalers::Crop => write!(f, "crop"),
            &Scalers::Distort => write!(f, "distort"),
            &Scalers::FitContain => write!(f, "fit_contain"),
            &Scalers::ForcedCover => write!(f, "forced_cover"),
        }
    }
}

/// The configuration struct for the image drawing.
///
/// *identifier* and *path* are the only required fields and will throw a panic if left empty.
///
/// By default *x* and *y* will be set to 0 and all other option will be set to None
///
/// ## Example
/// ```
/// use ueberzug::UeConf;
/// // The minimum required for proper config struct.
/// let conf = UeConf{
///             identifier:"carb",
///             path:"ferris.png",
///             ..Default::default()
///             };
///
/// // More specific option with starting x and y cordinates with width and height
/// let conf = UeConf{
///             identifier:"crab",
///             path:"ferris.png",
///             x:20,
///             y:5,
///             width:Some(30),
///             height:Some(30),
///             ..Default::default()
///             };
///```
pub struct UeConf {
    pub action: Actions,
    pub identifier: String,
    pub x: i16,
    pub y: i16,
    pub path: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub scaler: Option<Scalers>,
    pub draw: Option<bool>,
    pub synchronously_draw: Option<bool>,
    pub scaling_position_x: Option<f32>,
    pub scaling_position_y: Option<f32>,
}

impl Default for UeConf {
    fn default() -> Self {
        Self {
            action: Actions::Add,
            identifier: "".to_string(),
            x: 0,
            y: 0,
            path: "".to_string(),
            width: None,
            height: None,
            scaler: None,
            draw: None,
            synchronously_draw: None,
            scaling_position_x: None,
            scaling_position_y: None,
        }
    }
}

macro_rules! if_not_none {
    ($st:expr,$name:expr,$val:expr) => {
        match $val {
            Some(z) => $st + &format!(",\"{}\":\"{}\"", $name, z),
            None => $st,
        }
    };
}

impl UeConf {
    fn to_json(&self) -> String {
        if self.identifier == "" {
            panic!("Incomplete Information : Itentifier Not Found");
        }
        match self.action {
            Actions::Add => {
                if self.path == "" {
                    panic!("Incomplete Information : Path empty");
                }
                let mut jsn = String::from(r#"{"action":"add","#);
                jsn = jsn + &format!(
                    "\"path\":\"{}\",\"identifier\":\"{}\",\"x\":{},\"y\":{}",
                    self.path, self.identifier, self.x, self.y
                );
                jsn = if_not_none!(jsn, "width", self.width);
                jsn = if_not_none!(jsn, "height", self.height);
                jsn = if_not_none!(jsn, "scaler", self.scaler);
                jsn = if_not_none!(jsn, "draw", self.draw);
                jsn = if_not_none!(jsn, "sync", self.synchronously_draw);
                jsn = if_not_none!(
                    jsn,
                    "scaling_position_x",
                    self.scaling_position_x
                );
                jsn = if_not_none!(
                    jsn,
                    "scaling_position_y",
                    self.scaling_position_y
                );
                jsn = jsn + "}\n";
                jsn
            }
            Actions::Remove => format!(
                "{{\"action\":\"remove\",\"identifier\":\"{}\"}}\n",
                self.identifier
            ),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn enum_to_str() {
//         let add = Actions::Add;
//         let remove = Actions::Remove;
//         assert_eq!(add.to_string(), "add");
//         assert_eq!(format!("{}", remove), "remove");
//         let scaler_1 = Scalers::Contain;
//         let scaler_2 = Scalers::FitContain;
//         assert_eq!(scaler_1.to_string(), "contain");
//         assert_eq!(scaler_2.to_string(), "fit_contain");
//     }
//     #[test]
//     fn json_convertion() {
//         let conf = UeConf {
//             identifier: "a",
//             path: "a",
//             ..Default::default()
//         };
//         let rem_conf = UeConf {
//             action: Actions::Remove,
//             identifier: "a",
//             ..Default::default()
//         };
//         assert_eq!(
//             conf.to_json(),
//             "{\"action\":\"add\",\"path\":\"a\",\"identifier\":\"a\",\"x\":0,\"y\":0}\n"
//         );
//         assert_eq!(
//             rem_conf.to_json(),
//             "{\"action\":\"remove\",\"identifier\":\"a\"}\n"
//         );
//     }
// }
