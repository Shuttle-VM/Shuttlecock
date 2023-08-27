use occlum::{new_libos, ocall};
use occlum_ed25519::Signer;
use occlum_http_server::{HttpServer, HttpServerBuilder};
use occlum_sgxsdk::{SgxInitToken, SgxResult, SgxTimer};
use occlum_web3::{
    ContractABI, Eth, HttpTransport, JsonRpcClient, RawTransaction, Web3, TransactionReceipt,
};
use serde_json::json;
use std::{env, sync::mpsc::channel, thread, time::Duration};

fn main() -> SgxResult<()> {
    // Initialize the SGX enclave
    let token = SgxInitToken::default();
    let _enclave = token.initialize()?;

    // Load and initialize the smart contract
    let bytecode = include_str!("smart_contract.bytecode");
    let abi = include_str!("smart_contract.abi");
    let contract = ContractABI::load(&bytecode, &abi)?;
    
    // Scan the smart contract code with Mythril
    let mythril_output = ocall!(mythril_scan_smart_contract, bytecode)?;
    if mythril_output.contains("High risk issue found!") {
        panic!("Smart contract failed Mythril security scan");
    }

    // Start the HTTP server
    let transport = HttpTransport::new("http://127.0.0.1:8545")?;
    let client = JsonRpcClient::new(transport);
    let web3 = Web3::new(client);
    let eth = Eth::new(web3.clone());
    let tx_count = eth.transaction_count("0x0000000000000000000000000000000000000000", None)?;
    let signer = Signer::from("a4c2077c3f3d8f611c35a98e5bb5d54dc5f8f7c5e80b64c7e07d9b8c61702cc1");

    // Deploy the smart contract
    let tx = RawTransaction {
        nonce: tx_count,
        to: None,
        value: 0.into(),
        data: contract.clone().deploy(&signer)?,
        gas_price: 0.into(),
        gas: 2_000_000.into(),
        chain_id: 1.into(),
    };
    let signed_tx = signer.sign_raw_tx(tx)?;
    let tx_hash = eth.send_raw_transaction(signed_tx)?;

    // Wait for the contract to be mined
    let (tx_sender, rx) = channel();
    let timer = SgxTimer::new()?;
    timer.set_timeout(Duration::from_secs(60))?;
    thread::spawn(move || {
        let receipt = loop {
            match eth.transaction_receipt(&tx_hash) {
                Ok(receipt) => {
                    if let Some(receipt) = receipt {
                        break receipt;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get receipt: {:?}", e);
                }
            }
            thread::sleep(Duration::from_secs(1));
        };
        tx_sender.send(receipt).unwrap();
    });

    // Print the contract address
    let receipt = rx.recv_timeout(Duration::from_secs(60))?;
    if let Some(address) = receipt.contract_address {
        println!("Contract deployed at: {:?}", address);
    } else {
        panic!("Contract deployment failed");
    }

    Ok(())
}
