// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

use super::{
    storage_type::{
        validate_storage_address,
        Storage,
    },
    StorageAddress,
};

use crate::{
    client::{
        OfflineClientT,
        OnlineClientT,
    },
    error::Error,
    Config,
};
use derivative::Derivative;
use std::{
    future::Future,
    marker::PhantomData,
};

/// Query the runtime storage.
#[derive(Derivative)]
#[derivative(Clone(bound = "Client: Clone"))]
pub struct StorageClient<T, Client> {
    client: Client,
    _marker: PhantomData<T>,
}

impl<T, Client> StorageClient<T, Client> {
    /// Create a new [`StorageClient`]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

impl<T, Client> StorageClient<T, Client>
where
    T: Config,
    Client: OfflineClientT<T>,
{
    /// Run the validation logic against some storage address you'd like to access. Returns `Ok(())`
    /// if the address is valid (or if it's not possible to check since the address has no validation hash).
    /// Return an error if the address was not valid or something went wrong trying to validate it (ie
    /// the pallet or storage entry in question do not exist at all).
    pub fn validate<Address: StorageAddress>(
        &self,
        address: &Address,
    ) -> Result<(), Error> {
        validate_storage_address(address, &self.client.metadata())
    }
}

impl<T, Client> StorageClient<T, Client>
where
    T: Config,
    Client: OnlineClientT<T>,
{
    /// Obtain storage at some block hash.
    pub fn at(
        &self,
        block_hash: Option<T::Hash>,
    ) -> impl Future<Output = Result<Storage<T, Client>, Error>> + Send + 'static {
        // Clone and pass the client in like this so that we can explicitly
        // return a Future that's Send + 'static, rather than tied to &self.
        let client = self.client.clone();
        async move {
            // If block hash is not provided, get the hash
            // for the latest block and use that.
            let block_hash = match block_hash {
                Some(hash) => hash,
                None => {
                    client
                        .rpc()
                        .block_hash(None)
                        .await?
                        .expect("didn't pass a block number; qed")
                }
            };

            Ok(Storage::new(client, block_hash))
        }
    }
}
