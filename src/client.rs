// Copyright 2021 ChainSafe Systems
// SPDX-License-Identifier: LGPL-3.0-only

//! arweave client
use crate::{
    result::{Error, Result},
    types::{Block, FirehoseBlock, Transaction, ResponseRPC},
    Env,
};
use futures::future::join_all;
use rand::Rng;
use reqwest::{header::CONTENT_TYPE, Client as ReqwestClient, ClientBuilder};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use serde_json::{json, Value};

use std::time::Duration;

/// Arweave client
pub struct Client {
    client: ReqwestClient,
    /// arweave endpoints
    pub endpoints: Vec<String>,
    retry: u8,
}

// Define the RPC request structure
// #[derive(Serialize, Debug)]
// struct RpcRequest {
//     jsonrpc: &'static str,
//     id: u64,
//     method: &'static str,
//     params: Vec<Value>,
// }

// // Define the RPC response structure
// #[derive(Debug, Deserialize)]
// struct RpcResponse {
//     result: AstarBlock,
// }

impl Client {
    /// get next endpoint
    fn next_endpoint(&self) -> String {
        self.endpoints[rand::thread_rng().gen_range(0..self.endpoints.len())].to_string()
    }

    /// new arweave client
    pub fn new(endpoints: Vec<String>, timeout: Duration, retry: u8) -> Result<Self> {
        if endpoints.is_empty() {
            return Err(Error::EmptyEndpoints);
        }

        let client = ClientBuilder::new().gzip(true).timeout(timeout).build()?;

        Ok(Self {
            client,
            endpoints,
            retry,
        })
    }

    /// new client from environments
    pub fn from_env() -> Result<Self> {
        let env = Env::new()?;
        let client = ClientBuilder::new()
            .gzip(true)
            .timeout(Duration::from_millis(env.timeout))
            .build()?;

        Ok(Self {
            client,
            endpoints: env.endpoints,
            retry: env.retry,
        })
    }

    /// http get request with base url
    // async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
    //     let mut retried = 0;
    //     println!("{:#?}",self
    //         .client
    //         .get(&format!("{}/{}", self.next_endpoint(), path))
    //     );
    //     loop {
    //         match self
    //             .client
    //             .get(&format!("{}/{}", self.next_endpoint(), path))
    //             .send()
    //             .await?
    //             .json()
    //             .await
    //         {
    //             Ok(r) => return Ok(r),
    //             Err(e) => {
    //                 if retried < self.retry {
    //                     tokio::time::sleep(Duration::from_millis(1000)).await;
    //                     retried += 1;
    //                     continue;
    //                 }

    //                 return Err(e.into());
    //             }
    //         }
    //     }
    // }

    /// http post request with base url
    pub async fn get(&self, method: String, params: String) -> Result<Block> {
        println!("--- get_block --- {} {}", method, params);
        let url = "https://evm.astar.network/";
        // Define the RPC request parameters
        let input = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": method,
            "params": params,
        });
        // let json_data = format!(r#"{{"jsonrpc":"2.0", "id":"1", "method":"{}", "params":{}}}"#, method, params).as_str();
        let client = reqwest::Client::new();
        let response = self.client
            .post(url)
            .header(CONTENT_TYPE, "application/json;charset=utf-8")
            .body(input.to_string())
            .send()
            .await?
            .json::<ResponseRPC>()
            .await?;

        // Parse the response as JSON
        let result = response.result.clone();

        // Print the parsed JSON response
        println!("{:?}", result);
        println!("Hash: {:?}", result["hash"]);

        let block = serde_json::from_value::<Block>(result).unwrap();
        println!("Block {:?}", block);

        Ok(block)

    }



    fn build_request_json(&self, params: Value, method: &str) -> Value {
        // let jsonrpc = "2.0";
        json!({
           "jsonrpc": format!("2.0"),
           "id": format!("1"),
           "method": format!("{}", method),
           "params": params,
        })
    }

    // fn create_rpc_request_block() -> RpcRequest {
    //     RpcRequest {
    //         jsonrpc: "2.0",
    //         id: 1,
    //         method: "eth_getBlockByNumber",
    //         params: vec![
    //             json!("0x10126"), // Block number parameter
    //             json!(true),      // Include full transaction details
    //         ],
    //     }
    // }

    /// get arweave block by height
    ///
    /// ```rust
    /// use thegarii::types::Block;
    ///
    /// let client = thegarii::Client::from_env().unwrap();
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    ///
    /// { // block height 100 - https://arweave.net/block/height/100
    ///   let json = include_str!("../res/block_height_100.json");
    ///   let block = rt.block_on(client.get_block_by_height(100)).unwrap();
    ///   assert_eq!(block, serde_json::from_str::<Block>(&json).unwrap());
    /// }
    ///
    /// { // block height 269512 - https://arweave.net/block/height/269512
    ///   let json = include_str!("../res/block_height_269512.json");
    ///   let block = rt.block_on(client.get_block_by_height(269512)).unwrap();
    ///   assert_eq!(block, serde_json::from_str::<Block>(&json).unwrap());
    /// }
    ///
    /// { // block height 422250 - https://arweave.net/block/height/422250
    ///   let json = include_str!("../res/block_height_422250.json");
    ///   let block = rt.block_on(client.get_block_by_height(422250)).unwrap();
    ///   assert_eq!(block, serde_json::from_str::<Block>(&json).unwrap());
    /// }
    /// ```
    pub async fn get_block_by_height(&self, height: u64) -> Result<Block> {
        let params = format!(r#"[{}, true]"#, height.to_string());
        self.get("eth_getBlockByNumber".to_owned(), params).await
    }

    /// ```rust
    /// use thegarii::types::Block;
    ///
    /// let client = thegarii::Client::from_env().unwrap();
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    ///
    /// { //  using indep_hash of block_height_100
    ///   let json = include_str!("../res/block_height_100.json");
    ///   let hash = "ngFDAB2KRhJgJRysuhpp1u65FjBf5WZk99_NyoMx8w6uP0IVjzb93EVkYxmcErdZ";
    ///   let block = rt.block_on(client.get_block_by_hash(hash)).unwrap();
    ///   assert_eq!(block, serde_json::from_str::<Block>(&json).unwrap());
    /// }
    /// { //  using indep_hash of block_height_269512
    ///   let json = include_str!("../res/block_height_269512.json");
    ///   let hash = "5H-hJycMS_PnPOpobXu2CNobRlgqmw4yEMQSc5LeBfS7We63l8HjS-Ek3QaxK8ug";
    ///   let block = rt.block_on(client.get_block_by_hash(hash)).unwrap();
    ///   assert_eq!(block, serde_json::from_str::<Block>(&json).unwrap());
    /// }
    /// { //  using indep_hash of block_height_422250
    ///   let json = include_str!("../res/block_height_422250.json");
    ///   let hash = "5VTARz7bwDO4GqviCSI9JXm8_JOtoQwF-QCZm0Gt2gVgwdzSY3brOtOD46bjMz09";
    ///   let block = rt.block_on(client.get_block_by_hash(hash)).unwrap();
    ///   assert_eq!(block, serde_json::from_str::<Block>(&json).unwrap());
    /// }
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Block> {
        let params = format!(r#"[{}]"#, hash.to_string());
        self.get("eth_getBlockByHash".to_owned(), params).await    }

    /// get latest block
    pub async fn get_current_block(&self) -> Result<Block> {
        let params = format!(r#"[]"#);
        self.get("eth_blockNumber".to_owned(), params).await    }

    /// get arweave transaction by id
    ///
    /// ```rust
    /// use thegarii::types::Transaction;
    ///
    /// let client = thegarii::Client::from_env().unwrap();
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    ///
    /// { // tx BNttzDav3jHVnNiV7nYbQv-GY0HQ-4XXsdkE5K9ylHQ - https://arweave.net/tx/BNttzDav3jHVnNiV7nYbQv-GY0HQ-4XXsdkE5K9ylHQ
    ///   let json = include_str!("../res/tx.json");
    ///   let tx = rt.block_on(client.get_tx_by_id("BNttzDav3jHVnNiV7nYbQv-GY0HQ-4XXsdkE5K9ylHQ")).unwrap();
    ///   assert_eq!(tx, serde_json::from_str::<Transaction>(&json).unwrap());
    /// }
    /// ```
    pub async fn get_tx_by_id(&self, id: &str) -> Result<Transaction> {
        // self.get(&format!("tx/{}", id)).await
        todo!()
    }

    /// get arweave transaction data by id
    ///
    /// ```rust
    /// let client = thegarii::Client::from_env().unwrap();
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    ///
    /// { // tx BNttzDav3jHVnNiV7nYbQv-GY0HQ-4XXsdkE5K9ylHQ - https://arweave.net/tx/BNttzDav3jHVnNiV7nYbQv-GY0HQ-4XXsdkE5K9ylHQ/data
    ///   let json = include_str!("../res/data.json");
    ///   let tx = rt.block_on(client.get_tx_data_by_id("BNttzDav3jHVnNiV7nYbQv-GY0HQ-4XXsdkE5K9ylHQ")).unwrap();
    ///   assert_eq!(tx, json);
    /// }
    /// ```
    ///
    /// # NOTE
    ///
    /// timeout and retry don't work for this reqeust since we're not using
    /// this api in the polling service.
    pub async fn get_tx_data_by_id(&self, id: &str) -> Result<String> {
        Ok(self
            .client
            .get(&format!("{}/tx/{}/data", self.next_endpoint(), id))
            .send()
            .await?
            .text()
            .await?)
    }

    /// get and parse firehose blocks by height
    ///
    /// ```rust
    /// let client = thegarii::Client::from_env().unwrap();
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    ///
    /// { // block height 269512 - https://arweave.net/block/height/269512
    ///   let firehose_block = rt.block_on(client.get_firehose_block_by_height(269512)).unwrap();
    ///
    ///   let mut block_without_txs = firehose_block.clone();
    ///   block_without_txs.txs = vec![];
    ///
    ///   assert_eq!(block_without_txs, rt.block_on(client.get_block_by_height(269512)).unwrap().into());
    ///   for (idx, tx) in firehose_block.txs.iter().map(|tx| tx.id.clone()).enumerate() {
    ///     assert_eq!(firehose_block.txs[idx], rt.block_on(client.get_tx_by_id(&tx)).unwrap());
    ///   }
    /// }
    /// ```
    pub async fn get_firehose_block_by_height(&self, height: u64) -> Result<FirehoseBlock> {
        let block = self.get_block_by_height(height).await?;
        // println!("arweave block: {:?}", block);
        let txs: Vec<Transaction> = join_all(block.transactions.iter().map(|tx| self.get_tx_by_id(tx)))
            .await
            .into_iter()
            .collect::<Result<Vec<Transaction>>>()?;

        let mut firehose_block: FirehoseBlock = block.into();
        firehose_block.transactions = txs;
        Ok(firehose_block)
    }

    /// poll blocks from iterator
    ///
    /// ```rust
    /// let client = thegarii::Client::from_env().unwrap();
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    ///
    /// rt.block_on(client.poll(269512..269515)).unwrap();
    /// ```
    pub async fn poll<Blocks>(&self, blocks: Blocks) -> Result<Vec<FirehoseBlock>>
    where
        Blocks: Iterator<Item = u64> + Sized,
    {
        join_all(blocks.map(|block| self.get_firehose_block_by_height(block)))
            .await
            .into_iter()
            .collect::<Result<Vec<FirehoseBlock>>>()
    }
}
