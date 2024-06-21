use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct MarketPlacePermission {}

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
    }

    impl GenericMarketplace {
        pub fn start_marketplace(marketplace_fee: Decimal) -> (Global<GenericMarketplace>, Bucket) {
            let (marketplace_address_reservation, marketplace_component_address) =
                Runtime::allocate_component_address(GenericMarketplace::blueprint_id());

            // let global_caller_badge_rule =
            //     rule!(require(global_caller(marketplace_component_address)));

            let admin_key = ResourceBuilder::new_integer_non_fungible::<AdminKey>(OwnerRole::None)
                .mint_initial_supply([(1u64.into(), AdminKey {})]);

            let marketplace_listing_key =
                ResourceBuilder::new_integer_non_fungible::<MarketPlacePermission>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    })
                    .metadata(metadata! {
                        init {
                        "marketplace_fee" => marketplace_fee, updatable;
                        "marketplace_address" => marketplace_component_address, updatable;
                        }
                    })
                    .mint_initial_supply([(1u64.into(), MarketPlacePermission {})]);

            let key_manager =
                ResourceManager::from_address(marketplace_listing_key.resource_address());

            let component_address = Self {
                marketplace_listing_key_vault: Vault::with_bucket(marketplace_listing_key.into()),
                marketplace_key_manager: key_manager,
                marketplace_admin: admin_key.resource_manager(),
                marketplace_fee,
                fee_vaults: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
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
        ) {
            let nflid = NonFungibleLocalId::integer(1u64.into());
            let proof_creation: Proof = self
                .marketplace_listing_key_vault
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![nflid])
                .into();

            let mut fee: Vec<Bucket> = open_sale_address.call_raw::<Vec<Bucket>>(
                "purchase_royal_listing",
                scrypto_args!(nfgid, payment, proof_creation, account_recipient),
            );

            let fee_returned = fee.pop().unwrap();

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
        }

        pub fn get_marketplace_key_address(&self) -> ResourceAddress {
            self.marketplace_listing_key_vault.resource_address()
        }
    }
}
