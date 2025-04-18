# nectarflower-rs

A Rust client for interacting with Hive blockchain nodes via JSON-RPC, inspired by [nectarflower-go]. This library allows you to fetch node information from a Hive account's JSON metadata and make API calls to the Hive blockchain.

## Features

- Fetch node information from a Hive account's JSON metadata
- Filter out failing nodes
- Make JSON-RPC calls to Hive API endpoints
- Automatically retry failed calls on different nodes
- Idiomatic Rust API

## Installation

Add this crate to your `Cargo.toml`:

```toml
nectarflower-rs = "0.1.0"
```

## Usage

### Simplest Use Case: Just Get Passing Nodes

If you just need a list of passing Hive nodes for your own client implementation:

```rust
use nectarflower_rs::Client;

fn main() {
    // Create a client just to fetch nodes
    let client = Client::new();

    // Get nodes from account
    match client.get_nodes_from_account("nectarflower") {
        Ok(node_data) => {
            println!("Passing nodes:");
            for node in node_data.nodes {
                println!("{}", node);
            }
        },
        Err(e) => eprintln!("Error fetching nodes: {}", e),
    }
}
```

### Basic Usage

```rust
use nectarflower_rs::Client;
use serde_json::Value;

fn main() {
    // Create a new client with default node
    let mut client = Client::new();

    // Update nodes from account
    if let Err(e) = client.update_nodes_from_account("nectarflower") {
        eprintln!("Error updating nodes: {}", e);
        return;
    }

    // Make API calls using the updated client
    let props: Result<Value, _> = 
        client.call::<(), Value>("database_api.get_dynamic_global_properties", ());
    
    match props {
        Ok(props) => {
            // Get the block number
            if let Some(block_num) = props.get("head_block_number").and_then(|v| v.as_i64()) {
                println!("Current block number: {}", block_num);
            } else {
                println!("Current block number: {:?}", props.get("head_block_number"));
            }
        },
        Err(e) => eprintln!("Error fetching global properties: {}", e),
    }
}
```

### Advanced Usage

```rust
// Get nodes from account without updating the client
match client.get_nodes_from_account("nectarflower") {
    Ok(node_data) => {
        // Manually set nodes
        client.set_nodes(node_data.nodes, node_data.failing_nodes);
    },
    Err(e) => eprintln!("Error fetching nodes: {}", e),
}

// Make a custom API call
let accounts = vec!["nectarflower".to_string()];
let result: Result<Value, _> = 
    client.call("condenser_api.get_accounts", vec![accounts]);
```

### Fetching Block Data

```rust
// Create a client
let mut client = Client::new();

// Get the current block number
let props: Result<Value, _> = 
    client.call::<(), Value>("database_api.get_dynamic_global_properties", ());

// Get block number
let current_block_num = match props {
    Ok(props) => props.get("head_block_number").and_then(|v| v.as_i64()),
    Err(_) => None,
};

if let Some(block_num) = current_block_num {
    // Fetch a block that's a few blocks behind the head
    let target_block_num = block_num - 10;
    
    // Create parameters for get_block method
    let block_params = serde_json::json!({
        "block_num": target_block_num
    });

    // Fetch the block
    let block: Result<Value, _> = client.call("block_api.get_block", block_params);
    
    match block {
        Ok(block) => {
            // Extract block data
            if let Some(block_data) = block.get("block") {
                println!("Block ID: {}", 
                    block_data.get("block_id").and_then(|v| v.as_str()).unwrap_or("N/A"));
            }
        },
        Err(e) => eprintln!("Error fetching block: {}", e),
    }
}
```

## Examples

See the `examples/basic.rs` file for a complete example of how to use the library.

## License

MIT
