use scrypto::prelude::*;

// This blueprint is an example of a way to mint a new NFT collection. However, not all of this is required for a new collection and creators
// can choose to implement only the parts that are necessary for their use case. This blueprint combines the minting process including a reveal step
// to allow a creator to reveal the collection after minting, and a royalty payment vault/method to allow collection of royalties.
// It's possible that the minting process could be separated from the royalty payment process, but this blueprint combines them for simplicity.

#[derive(ScryptoSbor, NonFungibleData)]
struct Rascal {
    name: String,
    description: String,
    key_image_url: Url,
    attributes: Vec<HashMap<String, String>>,
}

#[derive(ScryptoSbor)] // To Do
pub enum RoyaltyFeeType {
    // No fee, standard Radix NFT settings
    None,
    // A flat fee is charged for each trade in selected currencies
    Flat,
    // A percentage fee is charged for each trade in any currencies (Recommended for most use cases)
    Percentage,
}

#[derive(ScryptoSbor)] // TO DO
pub enum PercentageFeeCurrencies {
    // Allow trading in any currency, such that the creator receives royalties in any currency. (Reccomended for most use cases)
    Any,
    // Allow trading in only selected currencies, such that the creator only receives royalties in those currencies.
    Selected,
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

        /// The royalty percentage to be paid to the creator of the Royal Rascals
        royalty_percent: Decimal,

        /// Royalty as a flat fee :: To Do
        royalty_flat: Decimal,

        /// Royalty maximum level :: To Do
        maximum_royalty_percent: Decimal,

        /// maximum royalty flat fee :: To Do
        maximum_royalty_flat: Decimal,

        /// lock royalty configuration
        royalty_configuration_locked: bool,

        /// All the royalty payments that have been made for different currencies
        royalty_vaults: KeyValueStore<ResourceAddress, Vault>,

        /// The address of the royalty component (which in this case, is this same component)
        royalty_component: ComponentAddress,

        /// Permissioned dApps - Dapps that you want to allow your NFTs to interact with/be deposited to.
        permissioned_dapps: KeyValueStore<ComponentAddress, ()>,
    }

    impl RoyalRascals {
        pub fn start_minting_rascals(
            mint_price: Decimal,
            mint_currency: ResourceAddress,
            collection_cap: u64,
            royalty_percent: Decimal,
            royalty_flat: Decimal,
            minimimum_royalty: Decimal,
            maximum_royalty_percent: Decimal,
            maximum_royalty_flat: Decimal,

            // Royalties are enforced for trading accounts, but the NFTs can only be used with specifically permissioned dApps.
            // (Recommended if royalties are part of revenue model)
            full_royalty_enforcement: bool,
            // Royalties are enforced for trading accounts, but the NFTs can be used with any other components that are not accounts.
            // (Recommended if you want higher interoperability)
            partial_royalty_enforcement: bool,
            // if both are false - this is configured as a standard Radix NFT, however royalties can be switched on at a later date.
            //
            // Option to lock the royalty configuration so that a creator can never change between full, partial, none royalty enforcement
            // or adjust the royalty fees.
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
                royalty_percent >= minimimum_royalty,
                "Royalty percent must be greater than minimum royalty"
            );

            assert!(
                royalty_percent <= maximum_royalty_percent,
                "Royalty percent must be less than maximum royalty"
            );

            assert!(
                royalty_flat <= maximum_royalty_flat,
                "Royalty flat fee must be less than maximum royalty flat fee"
            );

            assert!(
                royalty_flat >= minimimum_royalty,
                "Royalty flat fee must be greater than minimum royalty"
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
                        depositor => depositer_admin_rule;
                        depositor_updater => global_caller_badge_rule.clone();
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
                royalty_percent,
                royalty_flat: Decimal::from(0),
                maximum_royalty_percent: Decimal::from(1),
                maximum_royalty_flat: Decimal::from(0),
                royalty_configuration_locked,
                royalty_vaults: KeyValueStore::new(),
                permissioned_dapps: KeyValueStore::new(),
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
                    payment_amount.checked_mul(self.royalty_percent).unwrap(),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );
                self.royalty_vaults
                    .insert(currency.clone(), Vault::with_bucket(royalty));
            } else {
                // check the correct amount has been passed
                let royalty = payment.take_advanced(
                    payment_amount.checked_mul(self.royalty_percent).unwrap(),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );
                self.royalty_vaults.get_mut(&currency).unwrap().put(royalty);
            }

            // payment minus royalty returned to the trading account that called this method
            payment
        }

        pub fn lock_royalty_configuration(&mut self) {
            self.royalty_configuration_locked = true;
        }

        pub fn change_royalty_percentage_fee(&mut self, new_royalty_percent: Decimal) {
            assert!(
                !self.royalty_configuration_locked,
                "Royalty configuration is locked"
            );

            assert!(
                new_royalty_percent <= self.maximum_royalty_percent,
                "New royalty percentage is greater than maximum allowed"
            );

            self.royalty_percent = new_royalty_percent;
        }

        pub fn change_royalty_flat_fee(&mut self, new_royalty_flat: Decimal) {
            assert!(
                !self.royalty_configuration_locked,
                "Royalty configuration is locked"
            );

            assert!(
                new_royalty_flat <= self.maximum_royalty_flat,
                "New royalty flat fee is greater than maximum allowed"
            );

            self.royalty_flat = new_royalty_flat;
        }

        pub fn switch_royalty_to_flat_fee(&mut self, new_royalty_flat: Decimal) {
            assert!(
                !self.royalty_configuration_locked,
                "Royalty configuration is locked"
            );

            assert!(
                new_royalty_flat <= self.maximum_royalty_flat,
                "New royalty flat fee is greater than maximum allowed"
            );

            self.royalty_percent = Decimal::from(0);
            self.royalty_flat = new_royalty_flat;
        }

        pub fn switch_royalty_to_percentage_fee(&mut self, new_royalty_percent: Decimal) {
            assert!(
                !self.royalty_configuration_locked,
                "Royalty configuration is locked"
            );

            assert!(
                new_royalty_percent <= self.maximum_royalty_percent,
                "New royalty percentage is greater than maximum allowed"
            );

            self.royalty_percent = new_royalty_percent;
            self.royalty_flat = Decimal::from(0);
        }

        pub fn update_royalty_level(&mut self, new_royalty_level: String) {
            assert!(
                !self.royalty_configuration_locked,
                "Royalty configuration is locked"
            );

            // self.rascal_manager
            //     .update_metadata("royalty_level", new_royalty_level);
        }

        /// possibility to transfer the royalty NFT to a dApp if permissions are set for Full royalty enforcement
        /// Allow any dapp if royalties set to partial.
        /// To do - Remove this method here and put it on the trader account component. Unneccessary to confuse logic here.
        pub fn transfer_royalty_nft_to_dapp(
            &mut self,
            nft: Bucket,
            dapp: ComponentAddress,
            custom_method: String,
        ) {
            assert!(
                self.permissioned_dapps.get(&dapp).is_some(),
                "This dApp has not been permissioned by the collection creator"
            );

            let call_address: Global<AnyComponent> = Global(ObjectStub::new(
                ObjectStubHandle::Global(GlobalAddress::from(dapp)),
            ));

            let manfiest_method: &str = &custom_method;

            self.rascal_manager.set_depositable(rule!(allow_all));

            // send nft to dapp
            call_address.call_raw::<()>(manfiest_method, scrypto_args!(nft));

            self.rascal_manager.set_depositable(rule!(
                require_amount(1, self.depositer_admin)
                    || require(global_caller(self.royalty_component))
            ));
        }
    }
}
