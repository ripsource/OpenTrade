use scrypto::prelude::*;

/// This blueprint is a trader account - where they can list items and where items are purchased from. Each method call the event emitter component.

#[derive(ScryptoSbor)]
struct Listing {
    secondary_seller_permissions: Vec<ResourceAddress>,
    currency: ResourceAddress,
    price: Decimal,
    vault: Vault,
}

#[blueprint]
mod opentrader {

    struct OpenTrader {
        auth_key: NonFungibleGlobalId,
        listings: KeyValueStore<NonFungibleGlobalId, Listing>,
        account_locker: Global<AccountLocker>,
        my_account: Global<Account>,
        virtual_badge: Vault,
    }

    impl OpenTrader {
        pub fn create_trader(
            auth_key: NonFungibleGlobalId,
            my_account: Global<Account>,
            virtual_badge: Bucket,
        ) -> Global<OpenTrader> {
            let (trader_address_reservation, trader_component_address) =
                Runtime::allocate_component_address(OpenTrader::blueprint_id());
            let global_caller_badge_rule = rule!(require(global_caller(trader_component_address)));

            let account_locker = Blueprint::<AccountLocker>::instantiate(
                OwnerRole::Updatable(global_caller_badge_rule.clone()),
                global_caller_badge_rule.clone(),
                global_caller_badge_rule.clone(),
                global_caller_badge_rule.clone(),
                global_caller_badge_rule.clone(),
                None,
            );

            Self {
                auth_key,
                listings: KeyValueStore::new(),
                account_locker,
                my_account,
                virtual_badge: Vault::with_bucket(virtual_badge),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(trader_address_reservation)
            .globalize()
        }

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
            self.check_creator(creator_key);

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
            self.check_creator(creator_key);

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

            self.account_locker
                .store(self.my_account, payment.into(), true);

            nft_bucket
        }

        pub fn check_creator(&self, creator_key: Proof) {
            let creator_key_checked = creator_key.check(self.auth_key.resource_address());

            let local_id = creator_key_checked
                .as_non_fungible()
                .non_fungible_local_id();

            let nfgid = NonFungibleGlobalId::new(self.auth_key.resource_address(), local_id);

            assert!(self.auth_key == nfgid, "Creator key does not match");
        }
    }
}
