use scrypto::prelude::*;

#[blueprint]
mod generic_dapp {

    struct Dapp {
        vaults: KeyValueStore<ResourceAddress, Vault>,
    }

    impl Dapp {
        pub fn start_dapp() -> Global<Dapp> {
            Self {
                vaults: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn deposit_royalty_nft(&mut self, royalty_nft: Bucket) -> Option<Vec<Bucket>> {
            // insert the NFT into you dapp where needed
            // this method will be authorised to deposit the NFT into a vault within the component
            // so you can perform any logic you want here.
            // the method could have any arguments you like, all you need to do is pass the method name and
            // component address to the trader account method for transfer_to_dapp.

            let resource_address = royalty_nft.resource_address();
            self.vaults
                .insert(resource_address, Vault::with_bucket(royalty_nft));

            // process the deposit with some logic,
            //......
            //.....
            // create a badge, send some tokens, etc.
            // Or do nothing depending on the dApp (i.e. nft loans, gaming dApp, etc.)

            // option to return a vector of buckets (containing user badges, tokens, etc.) if required.
            // Some(vec![bucket1, bucket2, bucket3]), etc.
            None
        }

        pub fn withdraw_royalty_nft(
            &mut self,
            resource_address: ResourceAddress,
            trader_account: ComponentAddress,
        ) -> Vec<Bucket> {
            // withdraw the NFT from your dapp
            // There's no restrictions on withdraws - however you would need to pass this method to a deposit method on the nft collection
            // to permitt the deposit.

            let vault = self.vaults.get_mut(&resource_address);

            let mut return_bucket: Vec<Bucket> = vec![];

            if let Some(mut vault) = vault {
                let nft = vault.take_all();

                let call_address: Global<AnyComponent> = Global(ObjectStub::new(
                    ObjectStubHandle::Global(GlobalAddress::from(trader_account)),
                ));

                let return_receipt: Bucket =
                    call_address.call_raw::<Bucket>("deposit_royalty_nft", scrypto_args!(nft));

                return_bucket.push(return_receipt);
            } else {
                panic!("NFT not found in vault");
            };

            return_bucket
        }
    }
}
