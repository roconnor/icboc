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

//! `getaddress`
//!
//! Gets information about data stored by the wallet
//!

use anyhow::Context;
use icboc::Dongle;
use serde::Deserialize;
use std::path::Path;

/// Gets/updates an address
pub struct GetNewAddress;

/// Gets information
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// Which descriptor to generate the address from
    descriptor: usize,
    /// Which index to use
    #[serde(default)]
    index: Option<u32>,
    /// Note to attach to the address
    note: String,
}

impl super::Command for GetNewAddress {
    type Options = Options;

    fn execute<D: Dongle, P: AsRef<Path>>(
        options: Self::Options,
        wallet_path: P,
        dongle: &mut D,
    ) -> anyhow::Result<()> {
        let (key, nonce) = super::get_wallet_key_and_nonce(dongle)?;
        let mut wallet = super::open_wallet(&mut *dongle, &wallet_path, key)?;

        let timestr = time::strftime("%F %T%z", &time::now()).unwrap();
        assert_eq!(timestr.bytes().len(), 24);

        if options.descriptor >= wallet.n_descriptors() {
            return Err(anyhow::Error::msg(format!(
                "no descriptor with index {}",
                options.descriptor
            )));
        }

        // FIXME should notice/warn when overwriting an existing address
        let addr = wallet
            .add_address(options.descriptor, options.index, timestr, options.note)
            .context("adding address")?;

        println!("{}", addr);

        super::save_wallet(&wallet, wallet_path, key, nonce)
            .with_context(|| "saving wallet after import")?;

        Ok(())
    }
}
