use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct State {
    #[serde(rename = "ociVersion")]
    pub version: String,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "pid", default)]
    pub pid: i32,
    #[serde(rename = "bundle")]
    pub bundle: String,
    #[serde(rename = "annotations")]
    pub annotations: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct ContainerProcessState {
    #[serde(rename = "ociVersion")]
    pub version: String,
    pub fds: Vec<String>,
    pub pid: i32,
    #[serde(default)]
    pub metadata: String,
    pub state: State
}