use std::collections::HashMap;
use std::error::Error;
use std::result::Result;
use std::sync::Arc;

use tch::*;
use tch::nn::*;

use super::gcs::*;
use super::network::*;

pub struct WeightsCache {
    weights_map : HashMap<String,Arc<(NetworkType,VarStore)>>,
}

impl WeightsCache {
    pub fn new() -> WeightsCache {
        WeightsCache { weights_map : HashMap::new() }
    }

    pub fn load_weights(&mut self, name:&str, network_type:NetworkType) -> Result<Arc<(NetworkType,VarStore)>, Box<dyn Error>> {
        Ok(self.weights_map.entry(name.to_string()).or_insert( {
            let path = format!("weights/{}", name);
            std::fs::create_dir_all("weights")?;
            download(&path,&path)?;

            let mut vs = VarStore::new(Device::Cpu);
            let _ = create_network(&vs.root(), network_type);
            vs.load(&path).unwrap();
            Arc::new((network_type,vs))
        }).clone())
    }
}
