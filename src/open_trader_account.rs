use scrypto::prelude::*;

/// This blueprint is a trader account - where they can list items and where items are purchased from. Each method calls the event emitter component.
/// A trader account has two sets of methods for listing and purchases - one for royalty enforced NFTs and one for standard NFTs.
/// The trader account stores a virtual badge that is used to authenticate event emitters from each trader account and allows traders to buy and sell Royalty NFTs
/// by providing authentication to the deposit rules on an Royalty NFT.
///
///
/// Currently AccountLockers are not used - however the ambition would be to add them so that a user does not have to claim their revenue manually.

#[derive(ScryptoSbor)]
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
    ///
    /// Because you can construct transactions atomically on Radix - you could technically list a Royalty NFT for 0 XRD,
    // Then in the same transaction, purchase the NFT to another account. This would be a way to send an NFT to another user without paying a royalty
    // potentially.

    // To combat this we can store a time on a listing of the exact second a listing was made. We then block users from purchasing
    // a listing within the same second it was listed. This would prevent the above scenario from happening during normal network usage
    // where transactions are processed in a few seconds. Idealy, we could get more granular than seconds, but this seems like a pragmatic
    // solution for now.
    time_of_listing: Instant,
}

#[blueprint]
mod opentrader {

    struct OpenTrader {
        /// The key value store of listings information for NFTs the user has listed for sale.
        listings: KeyValueStore<NonFungibleGlobalId, Listing>,
        /// The key value store of listings information for Royalty NFTs the user has listed for sale.
        royal_listings: KeyValueStore<NonFungibleGlobalId, Listing>,
        /// The key value store of vaults that store all the NFTs that the user has listed for sale.
        nft_vaults: KeyValueStore<NonFungibleGlobalId, Vault>,
        /// The key value store of vaults that store all the revenue the user has made from sales.
        /// This is used to store the revenue until the user claims it. However a future ambition is to use AccountLockers.
        /// Multiple currencies are supported.
        sales_revenue: KeyValueStore<ResourceAddress, Vault>,
        /// The royal admin badge that is used to authenticate deposits of Royalty NFTs.
        /// A user should never be able to withdraw this badge or access it in a unintended manner.
        royal_admin: Vault,
        /// The virtual badge that is used to authenticate event emitters from each trader account.
        /// A user should never be able to withdraw this badge or access it in a unintended manner.
        virtual_badge: Vault,
        /// The local id of the virtual badge that is used to authenticate event emitters from each trader account.
        virtual_badge_local: NonFungibleLocalId,
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
            virtual_badge: Bucket,
            depositer_admin: Bucket,
        ) -> Global<OpenTrader> {
            let (trader_address_reservation, trader_component_address) =
                Runtime::allocate_component_address(OpenTrader::blueprint_id());
            // let global_caller_badge_rule = rule!(require(global_caller(trader_component_address)));

            let (auth_key_resource, auth_key_local) = auth_key.into_parts();

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
                trader_account_component_address: trader_component_address,
                nft_vaults: KeyValueStore::new(),
                sales_revenue: KeyValueStore::new(),
                royal_admin: Vault::with_bucket(depositer_admin),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
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
            trader_badge: Proof,
        ) {
            // authenticate user
            self.check_creator(trader_badge);

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

            let new_listing = Listing {
                secondary_seller_permissions: permissions,
                currency,
                price,
                nfgid: nfgid.clone(),
                time_of_listing,
            };

            // add the listing information. We don't need to worry about
            // duplicating as a listing key entry is always removed when and NFT is sold
            // or if the listing is cancelled.
            self.royal_listings.insert(nfgid.clone(), new_listing);

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
            })
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

            // First authenticate the proof to check that the marketplace or private buyer has the correct permissions to purchase the NFT
            // We are just using a resource address as validation here - however this could be a more complex check in the future for local ids
            // so that for private deals a brand new resource doesn't need to be created.

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

                // We send the full payment to the nft so that it can take its %fee
                let mut remainder_after_royalty: Bucket =
                    Global::<AnyComponent>::from(call_address)
                        .call_raw("pay_royalty", scrypto_args!(nft_address, payment));

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
            self.royal_listings.remove(&nfgid);

            payment_buckets
        }

        /// Using the bottlenose update's ned owner_role assertion, we can ensure that a user can transfer an NFT to another account that they own
        /// without need to pay a royalty or fee.
        pub fn same_owner_account_transfer(
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
        /// This can only work if the Royalty NFT has selected the royalty enforcement level to be: Partial or if Full, the creator
        /// must have permissioned the dapp in their royalty component.
        /// Allowing transfers to components opens a lot of possibilities for the user to create new and interesting use cases
        /// however it also allows loopholes for avoiding royalties. The creator of a collection should be aware of this.
        /// We effectively turn off the restrictions for deposits, do some foreign method, then turn them back on so a
        /// dapp can do what they need to with the asset.
        /// We provide an optional return of a vector of buckets, which should cover most use cases.
        pub fn transfer_nft_to_component(
            &mut self,
            royalty_nft: Bucket,
            // the component of the dapp you want to transfer the NFT to
            component: Global<AnyComponent>,
            // the name of the method you want to use on this component (i.e. pub fn deposit, etc.)
            custom_method: String,
            // optional return vec of buckets for things like badges reciepts, etc. from the dapp
            // should we add the option to be able to send some other asset with the NFT to the dapp?
        ) -> Option<Vec<Bucket>> {
            let mut optional_return: Vec<Bucket> = vec![];

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

            // Each Royalty NFT has a set level of royalty enforcement - full or partial. So we first get this information
            // as well as the royalty component address.
            let royalty_nft_manager = ResourceManager::from_address(royalty_nft.resource_address());

            let royalty_level: String = royalty_nft_manager
                .get_metadata("royalty_level")
                .unwrap()
                .unwrap();

            let royalty_component_global_address: GlobalAddress = royalty_nft_manager
                .get_metadata("royalty_component")
                .unwrap()
                .unwrap();

            let royalty_component =
                ComponentAddress::new_or_panic(royalty_component_global_address.into());

            let call_address: Global<AnyComponent> = Global(ObjectStub::new(
                ObjectStubHandle::Global(GlobalAddress::from(royalty_component)),
            ));

            // if full enforcement is set then we need to send the asset to the royalty component first so that
            // we can check if the dapp has permission to receive the asset as set by the NFT creator.
            if royalty_level == "Full" {
                let existing_access_rule = rule!(
                    require_amount(1, self.royal_admin.resource_address())
                        || require(global_caller(royalty_component))
                );

                let nft_manager = ResourceManager::from_address(royalty_nft.resource_address());

                nft_manager.set_depositable(rule!(allow_all));

                let returned_buckets_full: Option<Vec<Bucket>> =
                    Global::<AnyComponent>::from(call_address).call_raw::<Option<Vec<Bucket>>>(
                        "transfer_to_component",
                        scrypto_args!(royalty_nft, custom_method.clone()),
                    );

                nft_manager.set_depositable(existing_access_rule);

                if returned_buckets_full.is_some() {
                    optional_return.extend(returned_buckets_full.unwrap());
                }

                // otherwise, we just send the nft onto the method that was input.
            } else if royalty_level == "Partial" {
                self.royal_admin.as_fungible().authorize_with_amount(1, || {
                    let returned_buckets: Option<Vec<Bucket>> =
                        Global::<AnyComponent>::from(call_address).call_raw::<Option<Vec<Bucket>>>(
                            "transfer_to_component",
                            scrypto_args!(royalty_nft, custom_method.clone()),
                        );

                    if returned_buckets.is_some() {
                        optional_return.extend(returned_buckets.unwrap());
                    }
                });
            }

            // return any optional buckets that were returned from the dapp
            Some(optional_return)
        }

        // When a dapp wants to send an NFT back to the user - they can use this method to deposit it back to the user.

        pub fn deposit_royalty_nft(&mut self, nft: Bucket) {
            self.royal_admin.as_fungible().authorize_with_amount(1, || {
                self.my_account.try_deposit_or_abort(nft.into(), None);
            });
        }

        // Non-Royalty Enforced Methods
        // I've not implemented these methods fully yet
        // lots to change up so can be ignored.
        // Overall - handling non-royalty NFTs is much simpler as there are no royalties to pay - so has not been
        // a priority to implement yet.

        pub fn list(
            &mut self,
            nft_bucket: Bucket,
            currency: ResourceAddress,
            price: Decimal,
            permissions: Vec<ResourceAddress>,
            trader_badge: Proof,
        ) {
            self.check_creator(trader_badge);

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

            let new_listing = Listing {
                secondary_seller_permissions: permissions,
                currency,
                price,
                nfgid: nfgid.clone(),
                time_of_listing,
            };

            self.nft_vaults
                .insert(nfgid.clone(), Vault::with_bucket(nft_bucket.into()));

            self.listings.insert(nfgid, new_listing);
        }

        pub fn revoke_market_permission(
            &mut self,
            nft_id: NonFungibleGlobalId,
            permission_id: ResourceAddress,
            trader_badge: Proof,
        ) {
            let trader_badge_checked =
                trader_badge.check_with_message(self.auth_key_resource, "Incorrect Badge Resource");

            let local_id = trader_badge_checked
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
            trader_badge: Proof,
        ) {
            self.check_creator(trader_badge);

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
            trader_badge: Proof,
        ) -> Vec<Bucket> {
            let trader_badge_checked =
                trader_badge.check_with_message(self.auth_key_resource, "Incorrect Badge Resource");

            let local_id = trader_badge_checked
                .as_non_fungible()
                .non_fungible_local_id();

            assert!(
                self.auth_key_local == local_id,
                "Creator key does not match"
            );

            let mut nft_bucket: Vec<Bucket> = vec![];

            {
                let mut nft = self
                    .nft_vaults
                    .get_mut(&nft_id)
                    .expect("[cancel] NFT not found");

                nft_bucket.push(nft.take_all());
            }

            self.listings.remove(&nft_id);

            nft_bucket
        }

        pub fn purchase_market_listing(
            &mut self,
            nfgid: NonFungibleGlobalId,
            payment: FungibleBucket,
            permission: Proof,
        ) -> Vec<Bucket> {
            let mut nft_bucket: Vec<Bucket> = vec![];

            {
                let listing = self
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

            self.listings.remove(&nfgid);

            // self.account_locker
            //     .store(self.my_account, payment.into(), true);

            nft_bucket
        }

        // utility methods

        pub fn check_creator(&self, trader_badge: Proof) {
            let trader_badge_checked =
                trader_badge.check_with_message(self.auth_key_resource, "Incorrect Badge Resource");

            // let trader_badge_checked = trader_badge.check(self.auth_key_resource);

            let local_id = trader_badge_checked
                .as_non_fungible()
                .non_fungible_local_id();

            assert!(
                self.auth_key_local == local_id,
                "Creator key does not match"
            );
        }

        pub fn fetch_auth_key(&self) -> (ResourceAddress, NonFungibleLocalId) {
            (self.auth_key_resource, self.auth_key_local.clone())
        }
    }
}
