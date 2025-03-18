use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct DashGroup {
    
}

impl DashGroup {
    pub fn new() -> Self {
        DashGroup {}
    }
}

impl Default for DashGroup {
    fn default() -> Self {
        DashGroup {
        }
    }
}