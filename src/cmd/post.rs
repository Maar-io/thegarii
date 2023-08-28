// Copyright 2021 ChainSafe Systems
// SPDX-License-Identifier: LGPL-3.0-only
use crate::{Client, Result};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Post {
    /// block number
    pub height: u64,
}

impl Post {
    pub async fn exec(&self) -> Result<()> {
        let client = Client::from_env()?;
        let block = client.post(self.height).await?;

        println!("{}", serde_json::to_string_pretty(&block)?);
        Ok(())
    }
}
