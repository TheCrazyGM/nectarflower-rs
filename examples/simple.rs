use nectarflower_rs::Client;

fn main() {
    // Create a client just to fetch nodes
    let client = Client::new();

    // Get nodes from account
    match client.get_nodes_from_account("nectarflower") {
        Ok(node_data) => {
            // Display passing nodes
            println!("Passing nodes:");
            for node in node_data.nodes {
                println!("{}", node);
            }
            
            // Display failing nodes
            println!("\nFailing nodes:");
            if node_data.failing_nodes.is_empty() {
                println!("None");
            } else {
                for (node, reason) in node_data.failing_nodes {
                    println!("{} - Reason: {}", node, reason);
                }
            }
        },
        Err(e) => eprintln!("Error fetching nodes: {}", e),
    }
}
