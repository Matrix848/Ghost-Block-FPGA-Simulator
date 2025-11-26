use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    InterruptProcessing,
    Open(PathBuf),
    OpenError(),
    Save(Option<PathBuf>),
}
