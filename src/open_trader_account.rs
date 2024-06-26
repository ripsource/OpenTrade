use crate::open_trade_event::event;
use scrypto::prelude::*;
/// This blueprint is a trader account - where they can list items and where items are purchased from. Each method calls the event emitter component.
/// A trader account has two sets of methods for listing and purchases - one for royalty enforced NFTs and one for standard NFTs.
/// The trader account stores a emitter badge that is used to authenticate event emitters from each trader account and allows traders to buy and sell Royalty NFTs
/// by providing authentication to the deposit rules on an Royalty NFT.
///
///
/// Currently AccountLockers are not used - however the ambition would be to add them so that a user does not have to claim their revenue manually.

#[derive(ScryptoSbor, Clone)]
pub struct Listing {
    /// The permissions that a secondary seller must have to sell an NFT. This is used to ensure that only selected
    /// marketplaces or private buyers can buy an NFT.
    secondary_seller_permissions: Vec<ResourceAddress>,
    /// The seller is able to decide what currency they want to sell their NFT in (e.g. XRD, FLOOP, EARLY, HUG)
    currency: ResourceAddress,
    /// The price of the NFT - this price will be subject to marketplace fees and creator royalties which are taken as a % of this amount.
    price: Decimal,
    /// The NFTGID being recorded is potentially redundant as it is the key of the listing in the listings key value store.
    /// The actual NFT is stored in the key value store of vaults separately.
    nfgid: NonFungibleGlobalId,
    /// trader's account address - helpful for aggregators to know where to fetch listings from.
    open_trader_account: ComponentAddress,
    ///
    /// Because you can construct transactions atomically on Radix - you could technically list a Royalty NFT for 0 XRD,
    // then in the same transaction, purchase the NFT to another account. This would be a way to send an NFT to another user without paying a royalty
    // potentially.

    // To combat this we can store a time on a listing of the exact second a listing was made. We then block users from purchasing
    // a listing within the same second it was listed. This would prevent the above scenario from happening during normal network usage
    // where transactions are processed in a few seconds. Idealy, we could get more granular than seconds, but this seems like a pragmatic
    // solution for now.
    time_of_listing: Instant,
}

// To Do: register types for the Listing struct and in other blueprints
#[blueprint]
mod opentrader {

    enable_method_auth! {
    roles {
        admin => updatable_by: [];
    },
    methods {
        list => restrict_to: [admin];
        royal_list => restrict_to: [admin];
        same_owner_royal_transfer => restrict_to: [admin];
        transfer_royal_nft_to_component => restrict_to: [admin];
        revoke_market_permission => restrict_to: [admin];
        add_buyer_permission => restrict_to: [admin];
        change_price => restrict_to: [admin];
        cancel_listing => restrict_to: [admin];
        cancel_royal_listing => restrict_to: [admin];
        purchase_royal_listing => PUBLIC;
        purchase_listing => PUBLIC;
        deposit_royalty_nft => PUBLIC;
        fetch_auth_key => PUBLIC;
    }
    }

    struct OpenTrader {
        /// The key value store of listings information for NFTs the user has listed for sale.
        listings: KeyValueStore<NonFungibleGlobalId, Listing>,
        /// The key value store of vaults that store all the NFTs that the user has listed for sale.
        nft_vaults: KeyValueStore<NonFungibleGlobalId, Vault>,
        /// The key value store of vaults that store all the revenue the user has made from sales.
        /// This is used to store the revenue until the user claims it. However a future ambition is to use AccountLockers.
        /// Multiple currencies are supported.
        sales_revenue: KeyValueStore<ResourceAddress, Vault>,
        /// The royal admin badge that is used to authenticate deposits of Royalty NFTs.
        /// A user should never be able to withdraw this badge or access it in a unintended manner.
        royal_admin: Vault,
        /// The emitter badge that is used to authenticate event emitters from each trader account.
        /// A user should never be able to withdraw this badge or access it in a unintended manner.
        emitter_badge: Vault,
        /// The local id of the emitter badge that is used to authenticate event emitters from each trader account.
        emitter_badge_local: NonFungibleLocalId,
        /// the central event emitter component that is used to emit events for all trades.
        event_manager: Global<event::Event>,
        /// The trading account badge resource address. This badge is held by the user and is used to authenticate methods on their trading account.
        auth_key_resource: ResourceAddress,
        /// The trading account badge local id. This badge is held by the user and is used to authenticate methods on their trading account.
        auth_key_local: NonFungibleLocalId,
        /// AccountLockers to be added
        // account_locker: Global<AccountLocker>,
        my_account: Global<Account>,
        /// This users trading account component address
        trader_account_component_address: ComponentAddress,
    }

    impl OpenTrader {
        /// creates a new trader account. This function should be called via the OpenTradeFactory component in order to be
        /// populated with the correct badges and permissions.
        pub fn create_trader(
            auth_key: NonFungibleGlobalId,
            my_account: Global<Account>,
            emitter_badge: Bucket,
            depositer_admin: Bucket,
            event_manager: Global<event::Event>,
        ) -> Global<OpenTrader> {
            let (trader_address_reservation, trader_component_address) =
                Runtime::allocate_component_address(OpenTrader::blueprint_id());
            // let global_caller_badge_rule = rule!(require(global_caller(trader_component_address)));

            let (auth_key_resource, auth_key_local) = auth_key.clone().into_parts();

            let emitter_badge_local = emitter_badge.as_non_fungible().non_fungible_local_id();

            Self {
                auth_key_local,
                auth_key_resource,
                listings: KeyValueStore::new(),
                // account_locker,
                my_account,
                emitter_badge: Vault::with_bucket(emitter_badge),
                emitter_badge_local,
                event_manager,
                trader_account_component_address: trader_component_address,
                nft_vaults: KeyValueStore::new(),
                sales_revenue: KeyValueStore::new(),
                royal_admin: Vault::with_bucket(depositer_admin),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => rule!(require(auth_key));
            ))
            .with_address(trader_address_reservation)
            .globalize()
        }

        //👑👑👑  Royalty Enforced Methods 👑👑👑 //

        /// Lists an NFT for sale by the user. The user provides the NFT, the price, the currency,
        /// and the ResourceAddress of a badge that a secondary seller must have to sell the NFT.
        /// We don't issue badges to Marketplaces, we just assume they have a badge that a user can easily select to mean
        /// they want to list on their marketplace. In reality, a user will likley just check a box for Trove, Foton and Radland, etc.
        /// and doesn't need to know the badge address.
        pub fn royal_list(
            &mut self,
            // The NFT to list for sale
            nft_to_list: Bucket,
            // The price of the NFT - this price will be subject to marketplace fees and creator royalties which are taken as a % of this amount.
            price: Decimal,
            // The currency the NFT is listed in
            currency: ResourceAddress,
            // The permissions that a secondary seller must have to sell an NFT. This is used to ensure that only selected
            // marketplaces or private buyers can buy an NFT.
            permissions: Vec<ResourceAddress>,
            // The badge that is used to authenticate the user listing the NFT
            // trader_badge: Proof,
        ) {
            // authenticate user
            // self.check_creator(trader_badge);

            assert!(
                price > Decimal::zero(),
                "[list_nft] Listing price must be greater than zero"
            );

            assert!(
                nft_to_list.amount() == dec!(1),
                "[list_nft] Only one NFT can be listed at a time"
            );

            // Gather data from the NFT to complete all the information needed to list the NFT

            let nft_address = nft_to_list.resource_address();

            let id = nft_to_list.as_non_fungible().non_fungible_local_id();

            let nfgid = NonFungibleGlobalId::new(nft_address, id.clone());

            // We take the time of the listing as seconds to prevent a user from listing and selling an NFT in the same second - i.e.
            // calling the list method and purchase method within the same transaction which could be used to send an NFT to another user for free
            // without any risk of someone sniping it.

            let time_of_listing = Clock::current_time_rounded_to_seconds();

            let open_trader_account = self.trader_account_component_address;

            let new_listing = Listing {
                secondary_seller_permissions: permissions,
                currency,
                price,
                nfgid: nfgid.clone(),
                open_trader_account,
                time_of_listing,
            };

            // add the listing information. We don't need to worry about
            // duplicating as a listing key entry is always removed when and NFT is sold
            // or if the listing is cancelled.
            self.listings.insert(nfgid.clone(), new_listing.clone());

            // As this is a royalty enforced listing, we need to use the royalty admin badge
            // to authenticate the deposit of the NFT.
            // As its not possible to delete vaults that are empty, we need to check if one has been
            // created for this NFT previously. If so, we just us the existing vault - otherwise, we create a new one.
            self.royal_admin.as_fungible().authorize_with_amount(1, || {
                let vault_exists = self.nft_vaults.get(&nfgid).is_some();

                if vault_exists {
                    let mut vault = self
                        .nft_vaults
                        .get_mut(&nfgid)
                        .expect("[royal_list] NFT not found");
                    vault.put(nft_to_list);
                } else {
                    self.nft_vaults
                        .insert(nfgid.clone(), Vault::with_bucket(nft_to_list));
                }
            });

            // finally we emit a listing event via the event emitter component
            let emitter_proof = self
                .emitter_badge
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);
            self.event_manager
                .listing_event(new_listing, nfgid.clone(), emitter_proof.into());
        }

        /// The intention is that in the majority of cases, a marketplace would call this method using their
        /// marketplace badge to authenticate the purchase, get the NFT and return it to the user on their platform.
        /// However, for a private deal, a user could call this method directly with a badge issued by the listing creator for this deal.
        pub fn purchase_royal_listing(
            &mut self,
            // The NFGID of the NFT to purchase
            nfgid: NonFungibleGlobalId,
            // The payment for the NFT
            payment: FungibleBucket,
            // The badge of the marketplace or private buyer that is purchasing the NFT
            permission: Proof,
            // The account that the NFT should be sent to
            mut account_recipient: Global<Account>,
        ) -> Vec<Bucket> {
            let mut payment_buckets = vec![];
            let listing_event: Listing;

            // First authenticate the proof to check that the marketplace or private buyer has the correct permissions to purchase the NFT
            // We are just using a resource address as validation here - however this could be a more complex check in the future for local ids
            // so that for private deals a brand new resource doesn't need to be created.

            let trading_permission = permission.resource_address();

            {
                let listing_permission = self
                    .listings
                    .get(&nfgid)
                    .expect("[purchase] Listing not found");

                assert!(
                    listing_permission
                        .secondary_seller_permissions
                        .contains(&trading_permission),
                    "[purchase] Marketplace does not have permission to purchase this listing"
                );
            }

            // We get the marketplace fee rate from the metadata of the proof
            // TO DO for a private sale, we need to wrap this step with a check otherwise it will panic for a private deal.

            let marketplace_fee_rate: Decimal = permission
                .skip_checking()
                .resource_manager()
                .get_metadata("marketplace_fee")
                .unwrap()
                .unwrap();

            // We calculate the marketplace fee from the payment amount.
            // This could be an unsafe decimal at this point - however when taking from the payment we use a safe rounding mode.

            let marketplace_fee = payment.amount().checked_mul(marketplace_fee_rate).unwrap();

            // We retrieve basic information about the listing, such as price, currency and time of the listing.
            {
                let listing = self
                    .listings
                    .get_mut(&nfgid)
                    .expect("[purchase] Listing not found");

                listing_event = listing.clone();

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

                // As mentioned elsewhere - we want to ensure no one can do an atomic transaction of listing and purchasing a Royalty NFT
                // as this would provide a loophole for trading NFTs without paying royalties. We do this by checking the time of the listing
                // and the time of the purchase. If they are the same, we abort the transaction.
                // Currently this is done to the second - however if there's is a more granular alternative, that would be prefferable.
                let time_of_listing = listing.time_of_listing;

                let time_of_purchase = Clock::current_time_rounded_to_seconds();

                assert!(
                    !time_of_purchase.compare(time_of_listing, TimeComparisonOperator::Eq),
                    "[purchase] Purchase made within the same second as listing is not allowed."
                );

                // We get the NFT from the vault

                let mut vault = self
                    .nft_vaults
                    .get_mut(&nfgid)
                    .expect("[purchase] NFT not found");

                let nft = vault.take_all().as_non_fungible();

                let nft_address = nft.resource_address();

                // We get the royalty component address from the NFT metadata
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

                // We send the full payment to the royalty component so that it can take its %fee.
                // We also provide the trading permission to check against any other permissions the creator has set.
                let mut remainder_after_royalty: Bucket =
                    Global::<AnyComponent>::from(call_address).call_raw(
                        "pay_royalty",
                        scrypto_args!(nft_address, payment, trading_permission),
                    );

                // we then take the marketplaces fee (we've already calculated this earlier based on the full payment amount).
                let marketplace_revenue = remainder_after_royalty.take_advanced(
                    marketplace_fee,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );

                payment_buckets.push(marketplace_revenue);

                // Sales revenue for the trader is then stored. In the future it would be good to utilise AccountLockers for better UX.
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

                // Finally we send the NFT to the account recipient

                self.royal_admin.as_fungible().authorize_with_amount(1, || {
                    account_recipient.try_deposit_or_abort(nft.into(), None);
                });
            }
            self.listings.remove(&nfgid);

            // finally we emit a listing event via the event emitter component
            let emitter_proof = self
                .emitter_badge
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

            self.event_manager
                .purchase_listing_event(listing_event, nfgid, emitter_proof.into());

            payment_buckets
        }

        pub fn cancel_royal_listing(&mut self, nfgid: NonFungibleGlobalId) {
            let mut nft_bucket: Vec<Bucket> = vec![];

            {
                let mut nft = self
                    .nft_vaults
                    .get_mut(&nfgid)
                    .expect("[cancel] NFT not found");

                nft_bucket.push(nft.take_all());
            }
            {
                let listing = self
                    .listings
                    .get(&nfgid)
                    .expect("[change_price] Listing not found");

                let emitter_proof = self
                    .emitter_badge
                    .as_non_fungible()
                    .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

                self.event_manager.cancel_listing_event(
                    listing.clone(),
                    nfgid.clone(),
                    emitter_proof.into(),
                );
            }

            self.listings.remove(&nfgid);

            self.royal_admin.as_fungible().authorize_with_amount(1, || {
                self.my_account
                    .try_deposit_or_abort(nft_bucket.pop().unwrap().into(), None);
            });
        }

        /// Using the bottlenose update's ned owner_role assertion, we can ensure that a user can transfer an NFT to another account that they own
        /// without need to pay a royalty or fee.
        pub fn same_owner_royal_transfer(
            &mut self,
            royalty_nft: Bucket,
            mut recipient: Global<Account>,
        ) {
            {
                // Getting the owner role of the account.
                let owner_role = recipient.get_owner_role();

                // Assert against it.
                Runtime::assert_access_rule(owner_role.rule);

                // Assertion passed - the caller is the owner of the account.
            }

            self.royal_admin.as_fungible().authorize_with_amount(1, || {
                recipient.try_deposit_or_abort(royalty_nft.into(), None);
            });
        }

        /// Transfers an NFT to a component. This method is used to transfer an NFT to a component that is not an account.
        /// This can only work if the Royalty NFT's configuration allows the dapp to receive the NFT. The NFT creator
        /// must have permissioned the dapp in their royalty component if they've chosen to turn on dapp limits.
        /// Allowing transfers to components opens a lot of possibilities for the user to create new and interesting use cases
        /// however it also allows loopholes for avoiding royalties. The creator of a collection should be aware of this.
        /// We effectively turn off the restrictions for deposits, do some foreign method, then turn them back on so a
        /// dapp can do what they need to with the asset.
        /// We provide an optional return of a vector of buckets, which should cover most use cases.
        pub fn transfer_royal_nft_to_component(
            &mut self,
            royalty_nft: Bucket,
            // the component of the dapp you want to transfer the NFT to
            component: Global<AnyComponent>,
            // the name of the method you want to use on this component (i.e. pub fn deposit, etc.)
            custom_method: String,
            // optional return vec of buckets for things like badges reciepts, etc. from the dapp
            // should we add the option to be able to send some other asset with the NFT to the dapp?
        ) -> Option<Vec<Bucket>> {
            // we get the package address of the component
            let package_address = component.blueprint_id().package_address;

            // we get the well-known package address of the account components
            let my_bech32_address =
                "package_rdx1pkgxxxxxxxxxaccntxxxxxxxxxx000929625493xxxxxxxxxaccntx";
            let global_account_address = PackageAddress::try_from_bech32(
                &AddressBech32Decoder::new(&NetworkDefinition::mainnet()),
                &my_bech32_address,
            )
            .unwrap();

            // check that we're not passing the asset to a global account address. This is important
            // to ensure someone isn't bypassing royalties by using this channel to send an NFT to another account.
            assert!(
                package_address != global_account_address,
                "Component can not be an account component"
            );

            // Each Royalty NFT has its royalty component addres in its top-level resource metadata
            let royalty_nft_manager = ResourceManager::from_address(royalty_nft.resource_address());

            let royalty_component_global_address: GlobalAddress = royalty_nft_manager
                .get_metadata("royalty_component")
                .unwrap()
                .unwrap();

            let royalty_component =
                ComponentAddress::new_or_panic(royalty_component_global_address.into());

            let call_address: Global<AnyComponent> = Global(ObjectStub::new(
                ObjectStubHandle::Global(GlobalAddress::from(royalty_component)),
            ));

            // We don't need to authorise anything here as deposits will be authorised from the royalty component.

            let returned_buckets_full: Option<Vec<Bucket>> =
                Global::<AnyComponent>::from(call_address).call_raw::<Option<Vec<Bucket>>>(
                    "transfer_to_component",
                    scrypto_args!(royalty_nft, custom_method.clone()),
                );

            returned_buckets_full
        }

        // When a dapp wants to send an NFT back to the user - they can use this method to deposit it back to the user.

        pub fn deposit_royalty_nft(&mut self, nft: Bucket) {
            self.royal_admin.as_fungible().authorize_with_amount(1, || {
                self.my_account.try_deposit_or_abort(nft.into(), None);
            });
        }

        //
        // General royalty/non-royalty related Methods //
        //

        pub fn list(
            &mut self,
            nft_bucket: Bucket,
            currency: ResourceAddress,
            price: Decimal,
            permissions: Vec<ResourceAddress>,
        ) {
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
                nft_bucket.as_non_fungible().non_fungible_local_id(),
            );

            let time_of_listing = Clock::current_time_rounded_to_seconds();

            let open_trader_account = self.trader_account_component_address;

            let new_listing = Listing {
                secondary_seller_permissions: permissions,
                currency,
                price,
                nfgid: nfgid.clone(),
                open_trader_account,
                time_of_listing,
            };

            self.nft_vaults
                .insert(nfgid.clone(), Vault::with_bucket(nft_bucket.into()));

            self.listings.insert(nfgid.clone(), new_listing.clone());

            // finally we emit a listing event via the event emitter component
            let emitter_proof = self
                .emitter_badge
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

            self.event_manager
                .listing_event(new_listing, nfgid, emitter_proof.into());
        }

        pub fn revoke_market_permission(
            &mut self,
            nft_id: NonFungibleGlobalId,
            permission_id: ResourceAddress,
        ) {
            let mut listing = self
                .listings
                .get_mut(&nft_id)
                .expect("[revoke_permission] Listing not found");

            listing
                .secondary_seller_permissions
                .retain(|permissions| permissions != &permission_id);

            let emitter_proof = self
                .emitter_badge
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

            self.event_manager
                .update_listing_event(listing.clone(), nft_id, emitter_proof.into());
        }

        pub fn add_buyer_permission(
            &mut self,
            nft_id: NonFungibleGlobalId,
            permission_id: ResourceAddress,
        ) {
            let mut listing = self
                .listings
                .get_mut(&nft_id)
                .expect("[add_permission] Listing not found");

            listing.secondary_seller_permissions.push(permission_id);

            let emitter_proof = self
                .emitter_badge
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

            self.event_manager
                .update_listing_event(listing.clone(), nft_id, emitter_proof.into());
        }

        pub fn change_price(&mut self, nft_id: NonFungibleGlobalId, new_price: Decimal) {
            let mut listing = self
                .listings
                .get_mut(&nft_id)
                .expect("[change_price] Listing not found");
            listing.price = new_price;

            let emitter_proof = self
                .emitter_badge
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

            self.event_manager
                .update_listing_event(listing.clone(), nft_id, emitter_proof.into());
        }

        pub fn cancel_listing(&mut self, nft_id: NonFungibleGlobalId) -> Vec<Bucket> {
            let mut nft_bucket: Vec<Bucket> = vec![];

            {
                let mut nft = self
                    .nft_vaults
                    .get_mut(&nft_id)
                    .expect("[cancel] NFT not found");

                nft_bucket.push(nft.take_all());
            }
            {
                let listing = self
                    .listings
                    .get(&nft_id)
                    .expect("[change_price] Listing not found");

                let emitter_proof = self
                    .emitter_badge
                    .as_non_fungible()
                    .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

                self.event_manager.cancel_listing_event(
                    listing.clone(),
                    nft_id.clone(),
                    emitter_proof.into(),
                );
            }

            self.listings.remove(&nft_id);

            nft_bucket
        }

        pub fn purchase_listing(
            &mut self,
            nfgid: NonFungibleGlobalId,
            payment: FungibleBucket,
            permission: Proof,
        ) -> Vec<Bucket> {
            let mut nft_bucket: Vec<Bucket> = vec![];
            let listing_event: Listing;

            {
                let listing = self
                    .listings
                    .get_mut(&nfgid)
                    .expect("[purchase] Listing not found");

                listing_event = listing.clone();

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

                {
                    let mut nft = self
                        .nft_vaults
                        .get_mut(&nfgid)
                        .expect("[cancel] NFT not found");

                    nft_bucket.push(nft.take_all());
                }
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

            // finally we emit a listing event via the event emitter component
            let emitter_proof = self
                .emitter_badge
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset![self.emitter_badge_local.clone()]);

            self.event_manager.purchase_listing_event(
                listing_event,
                nfgid.clone(),
                emitter_proof.into(),
            );

            self.listings.remove(&nfgid);

            // self.account_locker
            //     .store(self.my_account, payment.into(), true);

            nft_bucket
        }

        // utility methods

        pub fn fetch_auth_key(&self) -> (ResourceAddress, NonFungibleLocalId) {
            (self.auth_key_resource, self.auth_key_local.clone())
        }
    }
}
