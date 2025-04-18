//! nectarflower-rs: A Rust client for Hive JSON-RPC

use reqwest::blocking::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// --- Account/Node types for metadata extraction ---
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountParams {
    pub accounts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub json_metadata: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeData {
    pub nodes: Vec<String>,
    pub failing_nodes: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Client {
    pub nodes: Vec<String>,
    pub failing_nodes: HashMap<String, String>,
    http_client: HttpClient,
}

impl Client {
    /// Create a new Hive client with a default node
    pub fn new() -> Self {
        Self {
            nodes: vec!["https://api.hive.blog".to_string()],
            failing_nodes: HashMap::new(),
            http_client: HttpClient::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// Set the list of nodes, filtering out invalid or failing nodes
    pub fn set_nodes(&mut self, nodes: Vec<String>, failing_nodes: HashMap<String, String>) {
        let valid_nodes = nodes
            .into_iter()
            .filter(|node| !failing_nodes.contains_key(node) && url::Url::parse(node).is_ok())
            .collect();
        self.nodes = valid_nodes;
        self.failing_nodes = failing_nodes;
    }

    /// Make a JSON-RPC call to the Hive API
    pub fn call<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, String> {
        let mut last_err = None;
        for node in &self.nodes {
            match self.call_node::<P, R>(node, method, &params) {
                Ok(res) => return Ok(res),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| "No nodes available".to_string()))
    }

    fn call_node<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        node: &str,
        method: &str,
        params: &P,
    ) -> Result<R, String> {
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };
        let resp = self
            .http_client
            .post(node)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .map_err(|e| format!("Request error: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("Unexpected status code: {}", resp.status()));
        }
        let rpc: RpcResponse<Value> = resp.json().map_err(|e| format!("Decode error: {e}"))?;
        if let Some(err) = rpc.error {
            return Err(format!("RPC error: {} (code: {})", err.message, err.code));
        }
        match rpc.result {
            Some(val) => {
                serde_json::from_value(val).map_err(|e| format!("Result decode error: {e}"))
            }
            None => Err("No result in RPC response".to_string()),
        }
    }

    /// Fetch account JSON metadata and extract node information
    pub fn get_nodes_from_account(&self, account_name: &str) -> Result<NodeData, String> {
        let params = AccountParams {
            accounts: vec![account_name.to_string()],
        };
        let resp: AccountsResponse = self
            .call("database_api.find_accounts", params)
            .map_err(|e| format!("Error fetching account: {e}"))?;
        let account = resp
            .accounts
            .get(0)
            .ok_or_else(|| format!("Account '{account_name}' not found"))?;
        let json_metadata = &account.json_metadata;
        let metadata_obj: Value = serde_json::from_str(json_metadata)
            .map_err(|e| format!("Error parsing JSON metadata: {e}"))?;
        let mut node_data = NodeData::default();
        if let Some(nodes) = metadata_obj.get("nodes") {
            node_data.nodes = serde_json::from_value(nodes.clone())
                .map_err(|e| format!("Error parsing nodes: {e}"))?;
        } else {
            return Err("No nodes found in account metadata".to_string());
        }
        if let Some(failing_nodes) = metadata_obj.get("failing_nodes") {
            node_data.failing_nodes =
                serde_json::from_value(failing_nodes.clone()).unwrap_or_else(|e| {
                    eprintln!("Warning: error parsing failing_nodes: {e}");
                    HashMap::new()
                });
        }
        Ok(node_data)
    }

    /// Fetch nodes from an account and update the client
    pub fn update_nodes_from_account(&mut self, account_name: &str) -> Result<(), String> {
        let node_data = self.get_nodes_from_account(account_name)?;
        self.set_nodes(node_data.nodes, node_data.failing_nodes);
        Ok(())
    }
}

// Stubs for JSON-RPC request/response types
#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest<P> {
    pub jsonrpc: String,
    pub method: String,
    pub params: P,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcResponse<R> {
    pub jsonrpc: String,
    pub result: Option<R>,
    pub error: Option<RpcError>,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}
