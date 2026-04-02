// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// GENERIC CONTRACT COMMANDS — Call any smart contract function
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use crate::commands::contract_ops;
use crate::{print_error, print_info, print_success, ContractCommands};
use std::path::Path;

pub async fn handle(
    action: ContractCommands,
    rpc: &str,
    config_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ContractCommands::Call {
            wallet,
            contract,
            function,
            args,
            gas_limit,
            amount,
        } => {
            let args_vec: Vec<String> = args.unwrap_or_default();
            let gas = gas_limit;
            let amount_cil = amount.unwrap_or(0);

            print_info(&format!(
                "Calling {}.{}({}) on {}",
                contract,
                function,
                args_vec.join(", "),
                rpc
            ));

            match contract_ops::call_contract(
                &wallet, &contract, &function, args_vec, gas, amount_cil, rpc, config_dir,
            )
            .await
            {
                Ok(data) => {
                    print_success("Contract call successful!");

                    // Try to parse and display output
                    if let Some(result) = data.get("result") {
                        if let Some(output_str) = result.get("output").and_then(|v| v.as_str()) {
                            if let Ok(parsed) =
                                serde_json::from_str::<serde_json::Value>(output_str)
                            {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&parsed)
                                        .unwrap_or(output_str.to_string())
                                );
                            } else {
                                println!("{}", output_str);
                            }
                        }
                        if let Some(gas_used) = result.get("gas_used") {
                            print_info(&format!("Gas used: {}", gas_used));
                        }
                    }
                    if let Some(hash) = data.get("block_hash").and_then(|v| v.as_str()) {
                        print_info(&format!("Block hash: {}", hash));
                    }
                }
                Err(e) => {
                    print_error(&format!("{}", e));
                }
            }
        }

        ContractCommands::Query {
            contract,
            function,
            args,
        } => {
            let args_vec: Vec<String> = args.unwrap_or_default();
            print_info(&format!(
                "Querying {}.{}({})",
                contract,
                function,
                args_vec.join(", ")
            ));

            let client = reqwest::Client::new();
            let url = format!("{}/query-contract", rpc);
            let payload = serde_json::json!({
                "contract_address": contract,
                "function": function,
                "args": args_vec,
            });

            let resp = client.post(&url).json(&payload).send().await?;
            let data: serde_json::Value = resp.json().await?;

            if data["status"].as_str() == Some("success") {
                print_success("Query successful!");
                if let Some(result) = data.get("result") {
                    if let Some(output_str) = result.get("output").and_then(|v| v.as_str()) {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(output_str) {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&parsed)
                                    .unwrap_or(output_str.to_string())
                            );
                        } else {
                            println!("{}", output_str);
                        }
                    }
                }
            } else {
                let msg = data["msg"].as_str().unwrap_or("Unknown error");
                print_error(&format!("Query failed: {}", msg));
            }
        }
    }

    Ok(())
}
