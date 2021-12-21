use std::collections::HashMap;
use std::error::Error;
use std::result::Result;
use std::sync::Arc;

use tch::*;
use tch::nn::*;

use super::gcs::*;
use super::network2::*;

pub struct WeightsCache {
    weights_map : HashMap<String,Arc<VarStore>>
}

impl WeightsCache {
    pub fn new() -> WeightsCache {
        WeightsCache { weights_map : HashMap::new() }
    }

    pub fn load_weights(&mut self, name:&str) -> Result<Arc<VarStore>, Box<dyn Error>> {
        Ok(self.weights_map.entry(name.to_string()).or_insert( {
            let path = format!("weights/{}", name);
            std::fs::create_dir_all("weights")?;
            download(&path,&path)?;

            let mut vs = VarStore::new(Device::Cpu);
            let _ = TchNetwork::new(&vs.root());
            vs.load(&path).unwrap();
            Arc::new(vs)
        }).clone())
    }
}
