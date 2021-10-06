use std::collections::HashMap;
use std::error::Error;
use std::result::Result;
use std::sync::Arc;

use tensorflow::Graph;

use super::gcs::*;
use super::network::load_graph_from_file;

pub struct GraphCache {
    graphs : HashMap<String,Arc<Graph>>
}

impl GraphCache {
    pub fn new() -> GraphCache {
        GraphCache { graphs : HashMap::new() }
    }

    pub fn load_graph(&mut self, name:&str) -> Result<Arc<Graph>, Box<dyn Error>> {
        Ok(self.graphs.entry(name.to_string()).or_insert( {
            download(name,"model.pb")?;
            Arc::new(load_graph_from_file("model.pb")?)
        }).clone())
    }
}
