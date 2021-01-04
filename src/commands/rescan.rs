// ICBOC 3D
// Written in 2021 by
//   Andrew Poelstra <icboc@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! `rescan`
//!
//! Scans the blockchain for new transactions
//!

use anyhow::Context;
use crate::rpc;
use icboc::Dongle;
use miniscript::{Descriptor, DescriptorPublicKey};
use serde::Deserialize;
use std::path::Path;

/// Scans the blockchain for new transactions
pub struct Rescan;

/// Scans the blockchain for new transactions
#[derive(Deserialize)]
pub struct Options {
    start_from: Option<u64>,
}

impl super::Command for Rescan {
    type Options = Options;

    fn execute<D: Dongle, P: AsRef<Path>>(
        options: Self::Options,
        wallet_path: P,
        bitcoind: &rpc::Bitcoind,
        dongle: &mut D,
    ) -> anyhow::Result<()> {
        let (key, nonce) = super::get_wallet_key_and_nonce(dongle)?;
        let mut wallet = super::open_wallet(&wallet_path, key)?;

        let mut cache = wallet.script_pubkey_cache(&mut *dongle)
            .context("getting scriptpubkeys from wallet")?;

        let mut height = options.start_from.unwrap_or(wallet.block_height.saturating_sub(100));
        let mut max_height = bitcoind.getblockcount()
            .context("getting initial block count")?;

        println!("Scanning from block {}. Current height: {}", height, max_height);
        while height < max_height {
            let block = bitcoind.getblock(height)
                .with_context(|| format!("fetching block {}", height))?;

            if height > 0 && height % 1000 == 0 {
                wallet.block_height = height;
                super::save_wallet(&wallet, &wallet_path, key, nonce)
                    .with_context(|| format!("saving wallet at height {}", height))?;
                println!("Height {:7}: {} {:?}", height, block.block_hash(), std::time::Instant::now());
            }

            let (received, spent) = wallet.scan_block(&block, height, &mut cache)
                .with_context(|| format!("updating wallet from block {}", height))?;
            for txo in received {
                println!("received {}", txo);
            }
            for txo in spent {
                println!("spent {}", txo);
            }

            height += 1;
            if height == max_height {
                max_height = bitcoind.getblockcount().context("getting block count")?;
            }
        }
        wallet.block_height = height;

        super::save_wallet(&wallet, &wallet_path, key, nonce)
            .with_context(|| format!("saving wallet at height {}", height))?;

        return Ok(());
    }
}

