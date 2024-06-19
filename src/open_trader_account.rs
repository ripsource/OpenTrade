use scrypto::prelude::*;

/// This blueprint is a trader account - where they can list items and where items are purchased from. Each method call the event emitter component.

#[derive(ScryptoSbor)]
pub struct Listing {
    secondary_seller_permissions: Vec<ResourceAddress>,
    currency: ResourceAddress,
    price: Decimal,
    vault: Vault,
}

#[derive(ScryptoSbor)]
pub struct RoyalListing {
    secondary_seller_permissions: Vec<ResourceAddress>,
    currency: ResourceAddress,
    price: Decimal,
    nfgid: NonFungibleGlobalId,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct NFData {
    name: String,
    description: String,
    key_image_url: Url,
    attributes: Vec<HashMap<String, String>>,
    royalty_component: ComponentAddress,
}

#[blueprint]
mod opentrader {

    struct OpenTrader {
        auth_key_resource: ResourceAddress,
        auth_key_local: NonFungibleLocalId,
        listings: KeyValueStore<NonFungibleGlobalId, Listing>,
        royal_listings: KeyValueStore<NonFungibleGlobalId, RoyalListing>,
        // account_locker: Global<AccountLocker>,
        my_account: Global<Account>,
        virtual_badge: Vault,
        virtual_badge_local: NonFungibleLocalId,
        warehouse_address: ComponentAddress,
        nft_vaults: KeyValueStore<NonFungibleGlobalId, Vault>,
        sales_revenue: KeyValueStore<ResourceAddress, Vault>,
        royal_admin: Vault,
    }

    impl OpenTrader {
        pub fn create_trader(
            auth_key: NonFungibleGlobalId,
            my_account: Global<Account>,
            virtual_badge: Bucket,
            depositer_admin: Bucket,
        ) -> Global<OpenTrader> {
            let (trader_address_reservation, trader_component_address) =
                Runtime::allocate_component_address(OpenTrader::blueprint_id());
            let global_caller_badge_rule = rule!(require(global_caller(trader_component_address)));

            let (auth_key_resource, auth_key_local) = auth_key.into_parts();

            // The ambition is use AccountLockers here to take sales revenue in the future so that
            // users don't have to claim their revenue manually.

            // let account_locker = Blueprint::<AccountLocker>::instantiate(
            //     OwnerRole::Updatable(global_caller_badge_rule.clone()),
            //     global_caller_badge_rule.clone(),
            //     global_caller_badge_rule.clone(),
            //     global_caller_badge_rule.clone(),
            //     global_caller_badge_rule.clone(),
            //     None,
            // );

            let virtual_badge_local = virtual_badge.as_non_fungible().non_fungible_local_id();

            Self {
                auth_key_local,
                auth_key_resource,
                listings: KeyValueStore::new(),
                royal_listings: KeyValueStore::new(),
                // account_locker,
                my_account,
                virtual_badge: Vault::with_bucket(virtual_badge),
                virtual_badge_local,
                warehouse_address: trader_component_address,
                nft_vaults: KeyValueStore::new(),
                sales_revenue: KeyValueStore::new(),
                royal_admin: Vault::with_bucket(depositer_admin),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(trader_address_reservation)
            .globalize()
        }

        pub fn fetch_auth_key(&self) -> (ResourceAddress, NonFungibleLocalId) {
            (self.auth_key_resource, self.auth_key_local.clone())
        }
        // Royalty Enforced Methods

        pub fn royal_list(
            &mut self,
            nft_to_list: Bucket,
            price: Decimal,
            currency: ResourceAddress,
            permissions: Vec<ResourceAddress>,
            creator_key: Proof,
        ) {
            self.check_creator(creator_key);

            assert!(
                price > Decimal::zero(),
                "[list_nft] Listing price must be greater than zero"
            );

            assert!(
                nft_to_list.amount() == dec!(1),
                "[list_nft] Only one NFT can be listed at a time"
            );

            // Get royalty-enforced NFT clearing component address

            let nft_address = nft_to_list.resource_address();

            let id = nft_to_list.as_non_fungible().non_fungible_local_id();

            let nfgid = NonFungibleGlobalId::new(nft_address, id.clone());

            let new_listing = RoyalListing {
                secondary_seller_permissions: permissions,
                currency,
                price,
                nfgid: nfgid.clone(),
            };

            self.royal_listings.insert(nfgid.clone(), new_listing);

            self.royal_admin.as_fungible().authorize_with_amount(1, || {
                self.nft_vaults
                    .insert(nfgid.clone(), Vault::with_bucket(nft_to_list));
            })
        }

        pub fn purchase_royal_listing(
            &mut self,
            nfgid: NonFungibleGlobalId,
            payment: FungibleBucket,
            permission: NonFungibleProof,
            account_recipient: Global<Account>,
        ) -> Vec<Bucket> {
            let mut payment_buckets = vec![];

            {
                let trading_permission = permission.resource_address();

                let listing_permission = self
                    .royal_listings
                    .get(&nfgid)
                    .expect("[purchase] Listing not found");

                assert!(
                    listing_permission
                        .secondary_seller_permissions
                        .contains(&trading_permission),
                    "[purchase] Marketplace does not have permission to purchase this listing"
                );
            }

            let marketplace_fee_rate: Decimal = permission
                .skip_checking()
                .resource_manager()
                .get_metadata("marketplace_fee")
                .unwrap()
                .unwrap();

            let marketplace_fee = payment.amount().checked_mul(marketplace_fee_rate).unwrap();

            {
                let listing = self
                    .royal_listings
                    .get_mut(&nfgid)
                    .expect("[purchase] Listing not found");

                let price = listing.price;

                assert!(
                    payment.amount() == price,
                    "[purchase] Payment amount does not match listing price"
                );

                let currency = listing.currency;

                assert!(
                    payment.resource_address() == currency,
                    "[purchase] Payment currency does not match listing currency",
                );

                let mut vault = self
                    .nft_vaults
                    .get_mut(&nfgid)
                    .expect("[purchase] NFT not found");

                let nft = vault.take_all().as_non_fungible();

                let nft_address = nft.resource_address();

                let royalty_component_global_address: GlobalAddress =
                    ResourceManager::from_address(nft_address)
                        .get_metadata("royalty_component")
                        .unwrap()
                        .unwrap();

                let royalty_component =
                    ComponentAddress::new_or_panic(royalty_component_global_address.into());

                let call_address: Global<AnyComponent> = Global(ObjectStub::new(
                    ObjectStubHandle::Global(GlobalAddress::from(royalty_component)),
                ));

                let mut remainder_after_royalty: Bucket =
                    Global::<AnyComponent>::from(call_address).call_raw(
                        "pay_royalty",
                        scrypto_args!(nft, payment, account_recipient),
                    );

                let marketplace_revenue = remainder_after_royalty.take_advanced(
                    marketplace_fee,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );

                payment_buckets.push(marketplace_revenue);

                let sales_vault_exists = self.sales_revenue.get(&currency).is_some();

                if sales_vault_exists {
                    self.sales_revenue
                        .get_mut(&currency)
                        .unwrap()
                        .put(remainder_after_royalty);
                } else {
                    let sales_vault = Vault::with_bucket(remainder_after_royalty);
                    self.sales_revenue.insert(currency, sales_vault);
                }
                // self.account_locker
                //     .store(self.my_account, submit_royalty, true);
            }

            self.royal_listings.remove(&nfgid);

            payment_buckets
        }

        // Non-Royalty Enforced Methods

        pub fn market_list_nft(
            &mut self,
            nft_bucket: NonFungibleBucket,
            currency: ResourceAddress,
            price: Decimal,
            permissions: Vec<ResourceAddress>,
            creator_key: Proof,
        ) {
            self.check_creator(creator_key);

            assert!(!nft_bucket.is_empty(), "[list_nft] No NFT provided");

            assert!(
                price > Decimal::zero(),
                "[list_nft] Listing price must be greater than zero"
            );

            assert!(
                nft_bucket.amount() == dec!(1),
                "[list_nft] Only one NFT can be listed at a time"
            );

            let nfgid = NonFungibleGlobalId::new(
                nft_bucket.resource_address(),
                nft_bucket.non_fungible_local_id(),
            );

            let new_listing = Listing {
                secondary_seller_permissions: permissions,
                currency,
                price,
                vault: Vault::with_bucket(nft_bucket.into()),
            };

            self.listings.insert(nfgid, new_listing);
        }

        pub fn revoke_market_permission(
            &mut self,
            nft_id: NonFungibleGlobalId,
            permission_id: ResourceAddress,
            creator_key: Proof,
        ) {
            let creator_key_checked =
                creator_key.check_with_message(self.auth_key_resource, "Incorrect Badge Resource");

            let local_id = creator_key_checked
                .as_non_fungible()
                .non_fungible_local_id();

            assert!(
                self.auth_key_local == local_id,
                "Creator key does not match"
            );

            let mut listing = self
                .listings
                .get_mut(&nft_id)
                .expect("[revoke_permission] Listing not found");

            listing
                .secondary_seller_permissions
                .retain(|permissions| permissions != &permission_id);
        }

        pub fn add_market_permission(
            &mut self,
            nft_id: NonFungibleGlobalId,
            permission_id: ResourceAddress,
            creator_key: Proof,
        ) {
            self.check_creator(creator_key);

            let mut listing = self
                .listings
                .get_mut(&nft_id)
                .expect("[add_permission] Listing not found");

            listing.secondary_seller_permissions.push(permission_id);
        }

        pub fn change_price(&mut self, nft_id: NonFungibleGlobalId, new_price: Decimal) {
            let mut listing = self
                .listings
                .get_mut(&nft_id)
                .expect("[change_price] Listing not found");
            listing.price = new_price;
        }

        pub fn cancel_market_listing(
            &mut self,
            nft_id: NonFungibleGlobalId,
            creator_key: Proof,
        ) -> Vec<Bucket> {
            let creator_key_checked =
                creator_key.check_with_message(self.auth_key_resource, "Incorrect Badge Resource");

            let local_id = creator_key_checked
                .as_non_fungible()
                .non_fungible_local_id();

            assert!(
                self.auth_key_local == local_id,
                "Creator key does not match"
            );

            let mut nft_bucket: Vec<Bucket> = vec![];

            {
                let mut listing = self
                    .listings
                    .get_mut(&nft_id)
                    .expect("[cancel] Listing not found");

                let vault = &mut listing.vault;

                nft_bucket.push(vault.take_all());
            }

            self.listings.remove(&nft_id);

            nft_bucket
        }

        pub fn purchase_market_listing(
            &mut self,
            nfgid: NonFungibleGlobalId,
            payment: FungibleBucket,
            permission: Proof,
        ) -> Vec<NonFungibleBucket> {
            let mut nft_bucket: Vec<NonFungibleBucket> = vec![];

            {
                let mut listing = self
                    .listings
                    .get_mut(&nfgid)
                    .expect("[purchase] Listing not found");

                let price = listing.price;

                assert!(
                    payment.amount() == price,
                    "[purchase] Payment amount does not match listing price"
                );

                let currency = listing.currency;

                assert!(
                    payment.resource_address() == currency,
                    "[purchase] Payment currency does not match listing currency",
                );

                let vault = &mut listing.vault;

                nft_bucket.push(vault.take_all().as_non_fungible());
            }

            let marketplace = permission.resource_address();

            let listing_permission = self
                .listings
                .get(&nfgid)
                .expect("[purchase] Listing not found");

            assert!(
                listing_permission
                    .secondary_seller_permissions
                    .contains(&marketplace),
                "[purchase] Marketplace does not have permission to purchase this listing"
            );

            self.listings.remove(&nfgid);

            // self.account_locker
            //     .store(self.my_account, payment.into(), true);

            nft_bucket
        }

        pub fn check_creator(&self, creator_key: Proof) {
            let creator_key_checked =
                creator_key.check_with_message(self.auth_key_resource, "Incorrect Badge Resource");

            // let creator_key_checked = creator_key.check(self.auth_key_resource);

            let local_id = creator_key_checked
                .as_non_fungible()
                .non_fungible_local_id();

            assert!(
                self.auth_key_local == local_id,
                "Creator key does not match"
            );
        }
    }
}
