use scrypto::prelude::*;

/// Overview
// This blueprint is an example of a way to mint a new Royalty NFT collection, including a random mint/reveal process.
// However, not all of this is required for a new collection and creators can choose to implement only the parts that
// are necessary for their use case. This blueprint combines the minting process including a reveal step
// to allow a creator to reveal the collection after minting, and a royalty payment vault/method to allow collection of royalties.
// It's possible that the minting process could be separated from the royalty payment process, but this blueprint combines them for simplicity.

// For a minting process - its likely this could be made into a factory component for no-code creators - however anyone could bring their own
// component and just add in the deposit rules and resource top-level metadata required for the royalty system. In fact, some interesting
// opportunites are available for creators to design reactive traits/features based on the trading activity and interaction of components with their NFTs.

///
/// The royalty config struct holds all the settings a creator can modify in relation to royalties on their NFTs.
/// There are a bunch of options you can enable and fine tuning you can do - in general, I would expect launchpad platforms to offer some pre-made config options.
/// Then an advanced mode for creators to fine-tune their settings. It's important to note that once you have a basic understanding of the core features,
/// you can easily extend the functionality and add new features to the royalty system. As long as some basic principles are followed, it will still be
/// compatible with the rest of the OpenTrade system.

#[derive(ScryptoSbor)]
struct RoyaltyConfig {
    /// The royalty percentage to be paid to the creator of the Royal Rascals (e.g. 0.1 = 10% - maximum value is 1)
    royalty_percent: Decimal,
    /// The maximum royalty percentage that can be set - once set can not be increased. It can be decreased though.
    maximum_royalty_percent: Decimal,
    /// Offers an option for a creator to only allow trading of their assets in certain currencies (currencies selected in the permitted_currencies field)
    limit_currencies: bool,
    /// Currencies that the creator can receive royalties in/an NFT can be traded in (e.g. XRD)
    permitted_currencies: KeyValueStore<ResourceAddress, ()>,
    /// Set minimum fixed amounts of royalties for each permitted currency
    /// this is useful if a creator wants to allow private sales, but still ensure they receive royalties.
    minimum_royalties: bool,
    /// Minimum royalty amounts for each currency
    minimum_royalty_amounts: KeyValueStore<ResourceAddress, Decimal>,
    // Permissioned dApps - Dapps that you want to allow your NFTs to interact with/be deposited to.
    limit_dapps: bool,
    /// A permission list of components an NFT can be transferred to
    permissioned_dapps: KeyValueStore<ComponentAddress, ()>,
    /// This is useful because private traders could trade the NFTs without paying royalties, so this closes that loophole.
    /// However, this can be turned off if the creator wants to allow any trader to trade the NFTs. If a creator wants to allow private sales,
    /// but still receive royalties - they can set a minimum royalty amount for each currency.
    limit_buyers: bool,
    /// A permission list for marketplaces/individual buyers that can trade the NFTs
    /// This requires that a certain badge is shown by the buyer or marketplace in order to purchase an NFT.
    permissioned_buyers: KeyValueStore<ResourceAddress, ()>,
    /// lock royalty configuration: Option can give traders confidence that the royalty percentage/settings will not change.
    /// There's no method to undo this once set to true. However, right now creators can always take steps to make their
    /// royalties more relaxed even if locked - i.e. remove mininimum royalties, allow all buyers, etc.
    royalty_configuration_locked: bool,
}

#[derive(ScryptoSbor, NonFungibleData)]
struct Rascal {
    name: String,
    description: String,
    key_image_url: Url,
    attributes: Vec<HashMap<String, String>>,
}

#[blueprint]
mod royal_rascals {

    // To Do System Access Rules

    struct RoyalRascals {
        rascal_manager: ResourceManager,
        rascal_creator_admin_manager: ResourceManager,
        rascal_creator_admin: ResourceAddress,

        depositer_admin: ResourceAddress,

        /// The price to mint a Royal Rascal NFT
        mint_price: Decimal,

        /// The selected currrency for minting Royal Rascals, e.g. XRD
        mint_currency: ResourceAddress,

        /// The maximum number of Royal Rascals that can be minted
        collection_cap: u64,

        /// The current mint ID for integer NFTs minted
        mint_id: u64,

        /// The vault for storing mint payments
        mint_payments_vault: Vault,

        /// All the royalty payments that have been made for different currencies
        royalty_vaults: KeyValueStore<ResourceAddress, Vault>,

        /// The address of the royalty component (which in this case, is this same component)
        royalty_component: ComponentAddress,

        /// The creator royalty settings
        royalty_config: RoyaltyConfig,
    }

    impl RoyalRascals {
        pub fn start_minting_rascals(
            // generic minting inputs (could be any set up for minting the collection)
            mint_price: Decimal,
            mint_currency: ResourceAddress,
            collection_cap: u64,

            // Required to enable trader accounts to interact with royalty NFTs
            depositer_admin: ResourceAddress,

            // royalty settings input
            royalty_percent: Decimal,
            maximum_royalty_percent: Decimal,

            // These represent some advanced setting that creators can enable to heighten the level of royalty enforcement
            // and use to create new reactive/dynamic features for their NFTs.
            limit_buyers: bool,
            limit_currencies: bool,
            limit_dapps: bool,
            minimum_royalties: bool,

            // This is relevant for transfers of an NFT to a component/Dapp - not for trading the NFTs.
            permissioned_dapps_input: Vec<ComponentAddress>,

            // Only applicable if limit buyers is set to true
            permissioned_buyers_input: Vec<ResourceAddress>,

            // only applicable if you want to restrict the currencies that can be used to pay royalties
            restricted_currencies_input: Vec<ResourceAddress>,
            // if restricting the currencies you can then also add minimum amounts for how much royalty you should receive.
            // This is set so that if you require 20 XRD as a minimum, and your %fee is 10% - then atleast a 200 XRD sale would be required.
            minimum_royalty_amounts_input: HashMap<ResourceAddress, Decimal>,

            // (reccommend setting to false and later locking the configuration if desired)
            royalty_configuration_locked: bool,
        ) -> (Global<RoyalRascals>, FungibleBucket) {
            let (rascal_address_reservation, royalty_component_address) =
                Runtime::allocate_component_address(RoyalRascals::blueprint_id());

            assert!(
                royalty_percent <= Decimal::from(1),
                "Royalty percent must be less than 100%"
            );

            assert!(
                royalty_percent <= maximum_royalty_percent,
                "Royalty percent must be less than maximum royalty"
            );

            let permissioned_dapps: KeyValueStore<ComponentAddress, ()> = KeyValueStore::new();
            let permissioned_buyers: KeyValueStore<ResourceAddress, ()> = KeyValueStore::new();
            let permitted_currencies: KeyValueStore<ResourceAddress, ()> = KeyValueStore::new();
            let minimum_royalty_amounts: KeyValueStore<ResourceAddress, Decimal> =
                KeyValueStore::new();

            if limit_dapps {
                for component_address in permissioned_dapps_input {
                    permissioned_dapps.insert(component_address, ());
                }
            }

            if !limit_buyers {
                for resource_address in permissioned_buyers_input {
                    permissioned_buyers.insert(resource_address, ());
                }
            }

            if limit_currencies {
                for currency in restricted_currencies_input {
                    permitted_currencies.insert(currency, ());
                }
                for (currency, amount) in minimum_royalty_amounts_input {
                    minimum_royalty_amounts.insert(currency, amount);
                }
            }

            // create the royalty config
            let royalty_config = RoyaltyConfig {
                royalty_percent,
                maximum_royalty_percent,
                limit_currencies,
                permitted_currencies,
                minimum_royalty_amounts,
                permissioned_dapps,
                permissioned_buyers,
                minimum_royalties,
                limit_buyers,
                limit_dapps,
                royalty_configuration_locked,
            };

            // create the unique badge for the creator of the collection
            let rascal_creator_admin = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .mint_initial_supply(1);

            // create the rules for the creator of the collection
            let creator_admin_rule = rule!(require(rascal_creator_admin.resource_address()));

            // create the rules for the global caller badge
            let global_caller_badge_rule = rule!(require(global_caller(royalty_component_address)));

            // This is the key rule that allows trader accounts to trade royalty NFTs.
            // In this example, we're allowing the component and trader accounts to deposit Rascal NFTs.
            let depositer_admin_rule = rule!(
                require_amount(1, depositer_admin)
                    || require(global_caller(royalty_component_address))
            );

            let rascal_manager =
                ResourceBuilder::new_integer_non_fungible::<Rascal>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => creator_admin_rule.clone();
                    })
                    .burn_roles(burn_roles! {
                        burner => creator_admin_rule.clone();
                        burner_updater => creator_admin_rule.clone();
                    })
                    //**** REQUIRED FOR ROYALTY COMPATABILITY */
                    // This rule creates the restriction that stops the NFTs from being traded without a royalty payment.
                    // Only the royalty component can bypass this rule and trader accounts can bypass this rule.
                    // If a creator wishes to leave the system completey - they can update the rules via methods on this component.
                    .deposit_roles(deposit_roles! {
                        depositor => depositer_admin_rule.clone();
                        depositor_updater => depositer_admin_rule;
                    })
                    .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                        non_fungible_data_updater => creator_admin_rule.clone();
                        non_fungible_data_updater_updater => creator_admin_rule.clone();
                    })
                    .metadata(metadata! {
                        roles {
                            metadata_locker => creator_admin_rule.clone();
                            metadata_locker_updater => creator_admin_rule.clone();
                            metadata_setter => global_caller_badge_rule.clone();
                            metadata_setter_updater => creator_admin_rule;
                        },
                        init {
                            "name" => "Royal Rascals".to_owned(), updatable;
                            //**** REQUIRED FOR ROYALTY COMPATABILITY */
                            // We include the royalty component address in the NFTs top-level metadata.
                            // This is important as it means we don't need to programmatically find royalty components on the dApp.
                            // Instead we can dynamically find the component on the NFTs Resource metadata.
                            // It's important we don't place this component address on the individual NFTs because
                            // that would require us knowing the exact NFT Metadata structure to fetch/handle this data within Scrypto.
                            "royalty_component" => royalty_component_address, updatable;

                        }
                    })
                    .create_with_no_initial_supply();

            let component_adresss = Self {
                rascal_manager,
                royalty_component: royalty_component_address,
                rascal_creator_admin_manager: rascal_creator_admin.resource_manager(),
                rascal_creator_admin: rascal_creator_admin.resource_address(),
                depositer_admin,
                mint_price,
                mint_currency: mint_currency.clone(),
                collection_cap,
                mint_id: 0,
                mint_payments_vault: Vault::new(mint_currency),
                royalty_vaults: KeyValueStore::new(),
                royalty_config,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(rascal_address_reservation)
            .globalize();

            (component_adresss, rascal_creator_admin)
        }

        pub fn resource_address(&self) -> ResourceAddress {
            self.rascal_manager.address()
        }

        /// This function allows users to buy a preview of an NFT before it is minted. This acts as a mechanism for random minting.
        /// Users pay for the mint cost and only a certain limit set by the cap can be minted.
        /// After the desired number of NFTs have been minted, then the creator can update the metadata on all or some of the NFTs.
        pub fn mint_preview_nft(
            &mut self,
            mut payment: Bucket,
            mut account: Global<Account>,
        ) -> Vec<Bucket> {
            assert!(
                payment.amount() >= self.mint_price,
                "[Mint Preview NFT] : Insufficient funds to mint NFT"
            );
            assert!(
                payment.resource_address() == self.mint_currency,
                "[Mint Preview NFT] : Incorrect currency to mint NFT"
            );

            assert!(
                self.mint_id < self.collection_cap,
                "[Mint Preview NFT] : Collection cap reached"
            );

            self.mint_payments_vault.put(payment.take(self.mint_price));

            let rascal = Rascal {
                name: "Rascal".to_string(),
                description: "A mischievous little rascal".to_string(),
                key_image_url: Url::of("https://rascal.com/placeholder.png"),
                attributes: vec![],
            };

            let minted_edition = self
                .rascal_manager
                .mint_non_fungible(&NonFungibleLocalId::Integer(self.mint_id.into()), rascal);

            self.mint_id += 1;

            account.try_deposit_or_abort(minted_edition, None);

            // we return any change from the transaction and the preview NFT
            vec![payment]
        }

        // this function updates the metadata on an NFT that has already been minted to reveal the collection
        pub fn mint_reveal(
            &mut self,
            nft_id: NonFungibleLocalId,
            name: String,
            description: String,
            key_image_url: String,
            attributes: Vec<HashMap<String, String>>,
            rascal_creator_admin: Proof,
        ) {
            let checked_admin = rascal_creator_admin.check(self.rascal_creator_admin);

            checked_admin.authorize(|| {
                self.rascal_manager
                    .update_non_fungible_data(&nft_id, "name", name);
                self.rascal_manager
                    .update_non_fungible_data(&nft_id, "description", description);
                self.rascal_manager.update_non_fungible_data(
                    &nft_id,
                    "key_image_url",
                    Url::of(key_image_url),
                );
                self.rascal_manager
                    .update_non_fungible_data(&nft_id, "attributes", attributes);
            })
        }

        // This function can be called by trader accounts when an NFT from this collection is purchased.
        // It takes the payment and an option for an account to send the NFT to.
        // It uses the royalty percentage set by the creator to determine how much of the payment to take.
        // We use a keyvaluestore of vaults so that we can store multiple currencies.
        // We take the NFT as an argument so that we can determine at this point whether we want to enforce advanced royalties settings
        // where only an account component can own the NFT - in which case we just sent the NFT directly to the input account.
        // Otherwise, we send the NFT back to the trading account component, where a it could be sent on to another component.
        pub fn pay_royalty(
            &mut self,
            nft: ResourceAddress,
            mut payment: Bucket,
            buyer: ResourceAddress,
        ) -> Bucket {
            let payment_amount = payment.amount();

            // check the correct NFT for this royalty component has been passed
            assert!(
                nft == self.rascal_manager.address(),
                "[pay_royalty] Incorrect resource passed"
            );

            if self.royalty_config.limit_buyers {
                assert!(
                    self.royalty_config
                        .permissioned_buyers
                        .get(&buyer)
                        .is_some(),
                    "This buyer is not permissioned to trade this NFT"
                );
            }

            let currency = payment.resource_address();
            let limit_currencies = self.royalty_config.limit_currencies;

            if limit_currencies {
                assert!(
                    self.royalty_config
                        .permitted_currencies
                        .get(&currency)
                        .is_some(),
                    "This currency is not permitted for royalties"
                );
            }

            // send the royalty to the royalty vault

            let vault_exists = self.royalty_vaults.get(&currency).is_some();

            if !vault_exists {
                // check the correct amount has been passed
                let royalty = payment.take_advanced(
                    payment_amount
                        .checked_mul(self.royalty_config.royalty_percent)
                        .unwrap(),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );

                if limit_currencies {
                    if self.royalty_config.minimum_royalties {
                        let minimum_royalty = self
                            .royalty_config
                            .minimum_royalty_amounts
                            .get(&currency)
                            .unwrap();
                        assert!(
                            royalty.amount() >= minimum_royalty.clone(),
                            "Royalty amount is below the minimum required"
                        );
                    }
                }

                self.royalty_vaults
                    .insert(currency.clone(), Vault::with_bucket(royalty));
            } else {
                // check the correct amount has been passed
                let royalty = payment.take_advanced(
                    payment_amount
                        .checked_mul(self.royalty_config.royalty_percent)
                        .unwrap(),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );

                if limit_currencies {
                    if self.royalty_config.minimum_royalties {
                        let minimum_royalty = self
                            .royalty_config
                            .minimum_royalty_amounts
                            .get(&currency)
                            .unwrap();
                        assert!(
                            royalty.amount() >= minimum_royalty.clone(),
                            "Royalty amount is below the minimum required"
                        );
                    }
                }
                self.royalty_vaults.get_mut(&currency).unwrap().put(royalty);
            }

            // payment minus royalty returned to the trading account that called this method
            payment
        }

        /// Possibility to transfer the royalty NFT to a dApp if permissions are set for advanced royalty enforcement - requires the dApp to be permissioned - transfer occurs here.
        /// If the royalty config allows it, then any dApp can interact with the NFT.
        /// We allow an optional return of a vector of buckets which should cover most use cases for dApps.
        ///
        /// As long as the code remains relatively similar - developers can use this method to have some reactive logic for when their NFTs interact with certain dApps.
        pub fn transfer_royalty_nft_to_dapp(
            &mut self,
            nft: Bucket,
            dapp: ComponentAddress,
            custom_method: String,
        ) -> Option<Vec<Bucket>> {
            if self.royalty_config.limit_dapps {
                assert!(
                    self.royalty_config.permissioned_dapps.get(&dapp).is_some(),
                    "This dApp has not been permissioned by the collection creator"
                );
            }

            let call_address: Global<AnyComponent> = Global(ObjectStub::new(
                ObjectStubHandle::Global(GlobalAddress::from(dapp)),
            ));

            let manfiest_method: &str = &custom_method;

            self.rascal_manager.set_depositable(rule!(allow_all));

            // send nft to dapp
            let optional_returned_buckets =
                call_address.call_raw::<Option<Vec<Bucket>>>(manfiest_method, scrypto_args!(nft));

            self.rascal_manager.set_depositable(rule!(
                require_amount(1, self.depositer_admin)
                    || require(global_caller(self.royalty_component))
            ));

            optional_returned_buckets
        }

        //
        // These set of methods offer the ability for the creator modify their royalty settings.
        //

        /// Only possible if the royalty configuration is not locked
        /// New percentage fee must be below the maximum set.
        pub fn change_royalty_percentage_fee(&mut self, new_royalty_percent: Decimal) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );

            assert!(
                new_royalty_percent <= self.royalty_config.maximum_royalty_percent,
                "New royalty percentage is greater than maximum allowed"
            );

            self.royalty_config.royalty_percent = new_royalty_percent;
        }

        /// you can always lower the maximum royalty percentage - even if the configuration is locked.
        pub fn lower_maximum_royalty_percentage(&mut self, new_max_royalty_percent: Decimal) {
            assert!(
                new_max_royalty_percent >= self.royalty_config.royalty_percent,
                "New maximum royalty percentage is less than current royalty percentage"
            );

            self.royalty_config.maximum_royalty_percent = new_max_royalty_percent;
        }

        /// Only possible if the royalty configuration is not locked.
        /// You can always turn this setting off even if the configuration is locked.
        pub fn restrict_currencies_true(&mut self) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.limit_currencies = true;
        }

        pub fn restrict_currencies_false(&mut self) {
            self.royalty_config.limit_currencies = false;
        }

        // You can only add restricted currencies if the restricted currency setting is turned on.
        // You can add even if the configuration is locked.
        pub fn add_permitted_currency(&mut self, currency: ResourceAddress) {
            assert!(
                self.royalty_config.limit_currencies,
                "Restricted currency setting is not turned on"
            );
            self.royalty_config
                .permitted_currencies
                .insert(currency, ());
        }

        // You can only remove restricted currencies if the restricted currency setting is turned on.
        // You can't remove currencies if the configuration is locked.
        pub fn remove_permitted_currency(&mut self, currency: ResourceAddress) {
            assert!(
                self.royalty_config.limit_currencies,
                "Restricted currency setting is not turned on"
            );
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.permitted_currencies.remove(&currency);
        }

        // You can only set minimum royalty amounts if the restricted currency setting is turned on.
        // You can't set minimum amounts if the configuration is locked.
        pub fn set_minimum_royalty_amount(
            &mut self,
            currency: ResourceAddress,
            minimum_royalty_amount: Decimal,
        ) {
            assert!(
                self.royalty_config.limit_currencies,
                "Restricted currency setting is not turned on"
            );
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config
                .minimum_royalty_amounts
                .insert(currency, minimum_royalty_amount);
        }

        // You can only remove minimum royalty amounts if the restricted currency setting is turned on.
        // You can remove even if the configuration is locked.
        pub fn remove_minimum_royalty_amount(&mut self, currency: ResourceAddress) {
            assert!(
                self.royalty_config.limit_currencies,
                "Restricted currency setting is not turned on"
            );
            self.royalty_config
                .minimum_royalty_amounts
                .remove(&currency);
        }

        // Permissioned dapps settings only work with advanced royalty enforcement settings.
        // You can add even if the configuration is locked.
        pub fn add_permissioned_dapp(&mut self, dapp: ComponentAddress) {
            self.royalty_config.permissioned_dapps.insert(dapp, ());
        }

        // You can't remove dapps if the configuration is locked.
        pub fn remove_permissioned_dapp(&mut self, dapp: ComponentAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.permissioned_dapps.remove(&dapp);
        }

        // Permissioned buyers settings only work with advanced royalty enforcement settings.
        // You can always add more permissioned buyers even if the configuration is locked.
        pub fn add_permissioned_buyer(&mut self, buyer: ResourceAddress) {
            self.royalty_config.permissioned_buyers.insert(buyer, ());
        }

        // You can't remove buyers if the configuration is locked.
        pub fn remove_permissioned_buyer(&mut self, buyer: ResourceAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.permissioned_buyers.remove(&buyer);
        }

        // You can't change to deny_all buyers if the configuration is locked.
        pub fn deny_all_buyers(&mut self) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.limit_buyers = false;
        }

        // You can allow all buyers even if the configuration is locked
        pub fn allow_all_buyers(&mut self) {
            self.royalty_config.limit_buyers = true;
        }

        pub fn lock_royalty_configuration(&mut self) {
            self.royalty_config.royalty_configuration_locked = true;
        }
    }
}
