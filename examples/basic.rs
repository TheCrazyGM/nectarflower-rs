//! Example usage for nectarflower-rs
use nectarflower_rs::Client;
use serde_json::Value;

fn main() {
    // Create a new Hive client with default node
    let mut client = Client::new();
    println!("Default client initialized with: {:?}", client.nodes);

    // Account to fetch nodes from
    let account_name = "nectarflower";

    // Get nodes from account
    println!("\nFetching nodes from account {}...", account_name);
    match client.get_nodes_from_account(account_name) {
        Ok(node_data) => {
            println!("Found {} nodes in account metadata", node_data.nodes.len());
            println!("Nodes: {:?}", node_data.nodes);
            if !node_data.failing_nodes.is_empty() {
                println!(
                    "Found {} failing nodes in account metadata\nFailing nodes: {:?}",
                    node_data.failing_nodes.len(),
                    node_data.failing_nodes
                );
            }

            // Update client with new nodes
            println!("\nUpdating client with new nodes...");
            client.set_nodes(node_data.nodes.clone(), node_data.failing_nodes.clone());
            println!("Updated client initialized with: {:?}", client.nodes);
        }
        Err(e) => {
            eprintln!("Error fetching nodes: {e}");
            return;
        }
    }

    // Test the updated client with a simple query
    println!("\nTesting updated client with a query...");
    let props: Result<Value, _> =
        client.call::<(), Value>("database_api.get_dynamic_global_properties", ());
    match props {
        Ok(props) => {
            let block_num = props.get("head_block_number").and_then(|v| v.as_i64());
            if let Some(num) = block_num {
                println!("Query successful! Current block number: {}", num);
            } else {
                println!(
                    "Query successful! Current block number: {:?}",
                    props.get("head_block_number")
                );
            }
        }
        Err(e) => {
            eprintln!("Error fetching global properties: {e}");
            return;
        }
    }

    // Demonstrate the all-in-one function
    println!("\nDemonstrating the all-in-one UpdateNodesFromAccount function...");
    let mut new_client = Client::new();
    match new_client.update_nodes_from_account(account_name) {
        Ok(()) => println!(
            "One-step update complete. Client initialized with: {:?}",
            new_client.nodes
        ),
        Err(e) => {
            eprintln!("Error updating nodes: {e}");
            return;
        }
    }

    // Example: Fetch a recent block
    println!("\nFetching a recent block...");
    // First get the current block number
    let block_props: Result<Value, _> =
        client.call::<(), Value>("database_api.get_dynamic_global_properties", ());
    let current_block_num = match block_props {
        Ok(props) => props.get("head_block_number").and_then(|v| v.as_i64()),
        Err(_) => None,
    };

    if let Some(block_num) = current_block_num {
        // Fetch a block that's a few blocks behind the head to ensure it's available
        let target_block_num = block_num - 10;
        println!("Fetching block #{}", target_block_num);

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
                    // Print block details
                    println!("Block details:");
                    println!(
                        "  Block ID: {}",
                        block_data
                            .get("block_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A")
                    );
                    println!(
                        "  Previous: {}",
                        block_data
                            .get("previous")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A")
                    );
                    println!(
                        "  Timestamp: {}",
                        block_data
                            .get("timestamp")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A")
                    );

                    // Print transaction count
                    if let Some(transactions) =
                        block_data.get("transactions").and_then(|v| v.as_array())
                    {
                        println!("  Transaction count: {}", transactions.len());

                        // If there are transactions, print details of the first one
                        if !transactions.is_empty() {
                            let tx_ids =
                                block_data.get("transaction_ids").and_then(|v| v.as_array());
                            let tx_id = tx_ids
                                .and_then(|ids| ids.get(0))
                                .and_then(|id| id.as_str())
                                .unwrap_or("unknown");

                            println!("\nFirst transaction details:");
                            println!("  Transaction ID: {}", tx_id);

                            // Pretty print the first transaction
                            if let Some(tx) = transactions.get(0) {
                                let tx_json = serde_json::to_string_pretty(tx)
                                    .unwrap_or_else(|_| "Error formatting transaction".to_string());
                                println!("  Transaction data:\n{}", tx_json);
                            }
                        }
                    }
                } else {
                    eprintln!("Error extracting block data");
                }
            }
            Err(e) => eprintln!("Error fetching block: {e}"),
        }
    } else {
        eprintln!("Could not determine current block number");
    }
}
