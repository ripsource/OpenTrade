use scrypto::prelude::*;

// This blueprint is an example of a way to mint a new Royalty NFT collection, including a random mint/reveal process.
// However, not all of this is required for a new collection and creators can choose to implement only the parts that
// are necessary for their use case. This blueprint combines the minting process including a reveal step
// to allow a creator to reveal the collection after minting, and a royalty payment vault/method to allow collection of royalties.
// It's possible that the minting process could be separated from the royalty payment process, but this blueprint combines them for simplicity.

// For a minting process - its likely this could be made into a factory component for no-code creators - however anyone could bring their own
// component and just add in the deposit rules and resource top-level metadata required for the royalty system. In fact, some interesting
// opportunites are available for creators to design reactive traits/features based on the trading activity and interaction of components with their NFTs.

/// The royalty config struct holds all the settings a creator can modify in relation to royalties on their NFTs.
/// There are tonnes of options and fine tuning you can do - in general, I would expect set-up platforms to offer some pre-made config options.
/// Then an advanced mode for creators to fine-tune their settings.

#[derive(ScryptoSbor)]
struct RoyaltyConfig {
    /// The royalty percentage to be paid to the creator of the Royal Rascals (e.g. 0.1 = 10% - maximum value is 1)
    royalty_percent: Decimal,
    /// The maximum royalty percentage that can be set - once set can not be increased. It can be decreased though.
    maximum_royalty_percent: Decimal,
    /// Offers an option for a creator to only allow trading of their assets in certain currencies (currencies selected in the restricted_currencies field)
    restricted_currency_setting: bool,
    /// [Only applicable if restricted_currency_setting is turned on] Currencies that the creator can receive royalties in/an NFT can be traded in (e.g. XRD)
    restricted_currencies: KeyValueStore<ResourceAddress, ()>,
    /// [Only applicable if restricted_currency_setting is turned on] Set minimum fixed amounts of royalties for each permitted currency
    /// this is useful if a creator wants to allow private sales, but still ensure they receive royalties.
    minimum_royalty_amounts: KeyValueStore<ResourceAddress, Decimal>,
    /// [Only applicable if royalty-enforcement set to FULL] Permissioned dApps - Dapps that you want to allow your NFTs to interact with/be deposited to.
    permissioned_dapps: KeyValueStore<ComponentAddress, ()>,
    /// [Only applicable if royalty-enforcement set to FULL] A permission list for marketplaces/buyers that can trade the NFTs
    /// This requires that a certain badge is shown by the buyer or marketplace in order to purchase an NFT.
    permissioned_buyers: KeyValueStore<ComponentAddress, ()>,
    /// Full royalty enforcement is the most restrictive setting, with options for finer tuned control.
    full_royalty_enforcement: bool,
    /// Partial royalty enforcement is a less restrictive setting, allowing more flexibility for trading/currencies/dapps.
    partial_royalty_enforcement: bool,
    /// [Only applicable if royalty-enforcement set to FULL] By default full enforcement only allows permissioned buyers (such as marketplaces)
    /// to trade the NFTs. This is useful because private traders could trade the NFTs without paying royalties, so this closes that loophole.
    /// However, this can be turned off if the creator wants to allow any trader to trade the NFTs. If a creator wants to allow private sales,
    /// but still receive royalties - they can set a minimum royalty amount for each currency.
    allow_all_buyers: bool,
    /// lock royalty configuration: Option can give traders confidence that the royalty percentage/settings will not change.
    /// There's no method to undo this once set to true. However, permissioned buyers and dapps can still be updated - these can only be locked
    /// if partial_roylaty enforcement is set.
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
            mint_price: Decimal,
            mint_currency: ResourceAddress,
            collection_cap: u64,
            royalty_percent: Decimal,
            maximum_royalty_percent: Decimal,
            full_royalty_enforcement: bool,
            partial_royalty_enforcement: bool,
            royalty_configuration_locked: bool,
            // Required to enable trader accounts to interact with royalty NFTs
            depositer_admin: ResourceAddress,
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

            let royalty_level: String;

            if full_royalty_enforcement && partial_royalty_enforcement {
                panic!("Cannot have both full and partial royalty enforcement");
            }

            if full_royalty_enforcement {
                royalty_level = "Full".to_string();
            } else if partial_royalty_enforcement {
                royalty_level = "Partial".to_string();
            } else {
                royalty_level = "None".to_string();
            }

            let royalty_config = RoyaltyConfig {
                royalty_percent,
                maximum_royalty_percent,
                restricted_currency_setting: false,
                restricted_currencies: KeyValueStore::new(),
                minimum_royalty_amounts: KeyValueStore::new(),
                permissioned_dapps: KeyValueStore::new(),
                permissioned_buyers: KeyValueStore::new(),
                full_royalty_enforcement,
                partial_royalty_enforcement,
                allow_all_buyers: true,
                royalty_configuration_locked,
            };

            let rascal_creator_admin = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .mint_initial_supply(1);

            let creator_admin_rule = rule!(require(rascal_creator_admin.resource_address()));

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
                    // This rule creates the restriction that stops the NFTs from being traded without a royalty payment.
                    // Only the royalty component can bypass this rule and trader accounts can bypass this rule.
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
                            // We include the royalty component address in the NFTs top-level metadata.
                            // This is important as it means we don't need to programmatically find royalty components on the dApp.
                            // Instead we can dynamically find the component on the NFTs Resource metadata.
                            // It's important we don't place this component address on the individual NFTs because
                            // that would require us knowing the exact NFT Metadata structure to fetch/handle this data within Scrypto.
                            "name" => "Royal Rascals".to_owned(), updatable;
                            "royalty_component" => royalty_component_address, updatable;
                            "royalty_level" => royalty_level.to_owned(), updatable;

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
        // We take the NFT as an argument so that we can determine at this point whether we want to enforce full royalties
        // where only an account component can own the NFT - in which case we just sent the NFT directly to the input account.
        // Otherwise, we send the NFT back to the trading account component, where a it could be sent on to another component.
        pub fn pay_royalty(
            &mut self,
            nft: ResourceAddress,
            mut payment: Bucket,
            // mut account: Global<Account>,
        ) -> Bucket {
            let payment_amount = payment.amount();

            // check the correct proof has been passed
            assert!(
                nft == self.rascal_manager.address(),
                "[pay_royalty] Incorrect resource passed"
            );

            let currency = payment.resource_address();

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
                self.royalty_vaults.get_mut(&currency).unwrap().put(royalty);
            }

            // payment minus royalty returned to the trading account that called this method
            payment
        }

        /// Possibility to transfer the royalty NFT to a dApp if permissions are set for Full royalty enforcement - requires the dApp to be permissioned - transfer occurs here.
        /// If the metadata on the resource is set to 'partial' enforcement then any dApp can interact with the NFT - the transfer occurs directly from the trading account component.
        /// We allow an optional return of a vector of buckets which should cover most use cases for dApps.
        ///
        /// As long as the code remains relatively similar - developers can use this method to have some reactive logic for when their NFTs interact with certain dApps.
        pub fn transfer_royalty_nft_to_dapp(
            &mut self,
            nft: Bucket,
            dapp: ComponentAddress,
            custom_method: String,
        ) -> Option<Vec<Bucket>> {
            assert!(
                self.royalty_config.permissioned_dapps.get(&dapp).is_some(),
                "This dApp has not been permissioned by the collection creator"
            );

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

        pub fn set_royalty_level_to_full(&mut self) {
            self.royalty_config.full_royalty_enforcement = true;
            self.royalty_config.partial_royalty_enforcement = false;
        }

        pub fn set_royalty_level_to_partial(&mut self) {
            self.royalty_config.full_royalty_enforcement = false;
            self.royalty_config.partial_royalty_enforcement = true;
        }

        pub fn set_royalty_level_to_none(&mut self) {
            self.royalty_config.full_royalty_enforcement = false;
            self.royalty_config.partial_royalty_enforcement = false;
        }

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

        pub fn lower_maximum_royalty_percentage(&mut self, new_max_royalty_percent: Decimal) {
            assert!(
                new_max_royalty_percent >= self.royalty_config.royalty_percent,
                "New maximum royalty percentage is less than current royalty percentage"
            );

            self.royalty_config.maximum_royalty_percent = new_max_royalty_percent;
        }

        pub fn restrict_currencies(&mut self, restricted_currency_setting: bool) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.restricted_currency_setting = restricted_currency_setting;
        }

        pub fn add_restricted_currency(&mut self, currency: ResourceAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config
                .restricted_currencies
                .insert(currency, ());
        }

        pub fn remove_restricted_currency(&mut self, currency: ResourceAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.restricted_currencies.remove(&currency);
        }

        pub fn set_minimum_royalty_amount(
            &mut self,
            currency: ResourceAddress,
            minimum_royalty_amount: Decimal,
        ) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config
                .minimum_royalty_amounts
                .insert(currency, minimum_royalty_amount);
        }

        pub fn remove_minimum_royalty_amount(&mut self, currency: ResourceAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config
                .minimum_royalty_amounts
                .remove(&currency);
        }

        pub fn add_permissioned_dapp(&mut self, dapp: ComponentAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.permissioned_dapps.insert(dapp, ());
        }

        pub fn remove_permissioned_dapp(&mut self, dapp: ComponentAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.permissioned_dapps.remove(&dapp);
        }

        pub fn add_permissioned_buyer(&mut self, buyer: ComponentAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.permissioned_buyers.insert(buyer, ());
        }

        pub fn remove_permissioned_buyer(&mut self, buyer: ComponentAddress) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.permissioned_buyers.remove(&buyer);
        }

        pub fn deny_all_buyers(&mut self) {
            assert!(
                !self.royalty_config.royalty_configuration_locked,
                "Royalty configuration is locked"
            );
            self.royalty_config.allow_all_buyers = false;
        }

        pub fn allow_all_buyers(&mut self) {
            self.royalty_config.allow_all_buyers = true;
        }

        pub fn lock_royalty_configuration(&mut self) {
            self.royalty_config.royalty_configuration_locked = true;
        }
    }
}
