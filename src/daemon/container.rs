use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct State {
    #[serde(rename = "ociVersion")]
    version: String,
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "pid", default)]
    pid: i32,
    #[serde(rename = "bundle")]
    bundle: String,
    #[serde(rename = "annotations")]
    annotations: HashMap<String, String>
}

#[derive(Deserialize)]
pub struct ContainerProcessState {
    #[serde(rename = "ociVersion")]
    version: String,
    fds: Vec<String>,
    pid: i32,
    #[serde(default)]
    metadata: String,
    state: State
}