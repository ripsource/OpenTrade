use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct MarketPlacePermission {
    name: String,
}

#[derive(ScryptoSbor, NonFungibleData)]
struct AdminKey {}

#[blueprint]
mod generic_marketplace {

    struct GenericMarketplace {
        marketplace_listing_key_vault: Vault,
        marketplace_key_manager: ResourceManager,
        marketplace_admin: ResourceManager,
        marketplace_fee: Decimal,
        fee_vaults: KeyValueStore<ResourceAddress, Vault>,
        mint_fee: Decimal,
    }

    impl GenericMarketplace {
        pub fn start_marketplace(
            marketplace_fee: Decimal,
            mint_fee: Decimal,
        ) -> (Global<GenericMarketplace>, Bucket) {
            let (marketplace_address_reservation, marketplace_component_address) =
                Runtime::allocate_component_address(GenericMarketplace::blueprint_id());

            let global_caller_badge_rule =
                rule!(require(global_caller(marketplace_component_address)));

            let admin_key = ResourceBuilder::new_integer_non_fungible::<AdminKey>(OwnerRole::None)
                .mint_initial_supply([(1u64.into(), AdminKey {})]);

            let marketplace_listing_key =
                ResourceBuilder::new_integer_non_fungible::<MarketPlacePermission>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule;
                        minter_updater => rule!(deny_all);
                    })
                    .metadata(metadata! {
                        init {
                        "marketplace_fee" => marketplace_fee, updatable;
                        "marketplace_address" => marketplace_component_address, updatable;
                        }
                    })
                    .mint_initial_supply([(
                        1u64.into(),
                        MarketPlacePermission {
                            name: "Generic Marketplace".to_string(),
                        },
                    )]);

            let key_manager =
                ResourceManager::from_address(marketplace_listing_key.resource_address());

            let component_address = Self {
                marketplace_listing_key_vault: Vault::with_bucket(marketplace_listing_key.into()),
                marketplace_key_manager: key_manager,
                marketplace_admin: admin_key.resource_manager(),
                marketplace_fee,
                fee_vaults: KeyValueStore::new(),
                mint_fee,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .metadata(metadata! (
                roles {
                    metadata_setter => rule!(deny_all);
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "name" => "Trove".to_owned(), locked;
                    "description" => "Trove NFT marketplace".to_owned(), locked;
                    "dapp_definition" => marketplace_component_address, locked;
                    "icon_url" => Url::of("https://trove.tools/trove%20square.png"), locked;
                }
            ))
            .with_address(marketplace_address_reservation)
            .globalize();

            (component_address, admin_key.into())
        }

        pub fn purchase_royal_listing(
            &mut self,
            nfgid: NonFungibleGlobalId,
            payment: FungibleBucket,
            open_sale_address: Global<AnyComponent>,
            account_recipient: Global<Account>,
        ) -> Vec<Bucket> {
            let nflid = NonFungibleLocalId::integer(1u64.into());
            let proof_creation: Proof = self
                .marketplace_listing_key_vault
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![nflid])
                .into();

            let mut fee_and_receipt: (Vec<Bucket>, Vec<Bucket>) =
                open_sale_address.call_raw::<(Vec<Bucket>, Vec<Bucket>)>(
                    "purchase_royal_listing",
                    scrypto_args!(nfgid, payment, proof_creation, account_recipient),
                );

            let fee_returned = fee_and_receipt.0.pop().unwrap();

            let fee_resource = fee_returned.resource_address();

            let fee_vault_exists = self.fee_vaults.get(&fee_resource).is_some();

            if fee_vault_exists {
                self.fee_vaults
                    .get_mut(&fee_resource)
                    .unwrap()
                    .put(fee_returned);
            } else {
                let fee_vault = Vault::with_bucket(fee_returned);
                self.fee_vaults.insert(fee_resource, fee_vault);
            }

            fee_and_receipt.1
        }

        pub fn purchase_listing(
            &mut self,
            nfgid: NonFungibleGlobalId,
            payment: FungibleBucket,
            trader_account_address: Global<AnyComponent>,
        ) -> Vec<Bucket> {
            let nflid = NonFungibleLocalId::integer(1u64.into());
            let proof_creation: Proof = self
                .marketplace_listing_key_vault
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![nflid])
                .into();

            let mut fee_and_nft: (Vec<Bucket>, Vec<Bucket>) =
                trader_account_address.call_raw::<(Vec<Bucket>, Vec<Bucket>)>(
                    "purchase_listing",
                    scrypto_args!(nfgid, payment, proof_creation),
                );

            let fee_returned = fee_and_nft.1.pop().unwrap();

            let fee_resource = fee_returned.resource_address();

            let fee_vault_exists = self.fee_vaults.get(&fee_resource).is_some();

            if fee_vault_exists {
                self.fee_vaults
                    .get_mut(&fee_resource)
                    .unwrap()
                    .put(fee_returned);
            } else {
                let fee_vault = Vault::with_bucket(fee_returned);
                self.fee_vaults.insert(fee_resource, fee_vault);
            }

            fee_and_nft.0
        }

        pub fn purchase_preview_mint(
            &mut self,
            mut payment: Bucket,
            account: Global<Account>,
            preview_mint_address: Global<AnyComponent>,
        ) -> Vec<Bucket> {
            let fee_amount = payment.amount().checked_mul(self.mint_fee).unwrap();

            let fee =
                payment.take_advanced(fee_amount, WithdrawStrategy::Rounded(RoundingMode::ToZero));

            let fee_resource = fee.resource_address();
            let fee_vault_exists = self.fee_vaults.get(&fee_resource).is_some();

            if fee_vault_exists {
                self.fee_vaults.get_mut(&fee_resource).unwrap().put(fee);
            } else {
                let fee_vault = Vault::with_bucket(fee);
                self.fee_vaults.insert(fee_resource, fee_vault);
            }

            let nflid = NonFungibleLocalId::integer(1u64.into());
            let proof_creation: Proof = self
                .marketplace_listing_key_vault
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![nflid])
                .into();

            let receipt_and_change: Vec<Bucket> = preview_mint_address.call_raw::<Vec<Bucket>>(
                "purchase_preview_mint",
                scrypto_args!(payment, account, proof_creation),
            );

            receipt_and_change
        }

        pub fn get_marketplace_key_address(&self) -> ResourceAddress {
            self.marketplace_listing_key_vault.resource_address()
        }
    }
}
