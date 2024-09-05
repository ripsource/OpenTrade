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

// #[derive(ScryptoSbor)]
// struct RoyaltyConfig {
//     /// The royalty percentage to be paid to the creator of the Royal NFTs (e.g. 0.1 = 10% - maximum value is 1)
//     royalty_percent: Decimal,
//     /// The maximum royalty percentage that can be set - once set can not be increased. It can be decreased though.
//     maximum_royalty_percent: Decimal,
//     /// Offers an option for a creator to only allow trading of their assets in certain currencies (currencies selected in the permitted_currencies field)
//     limit_currencies: bool,
//     /// Currencies that the creator can receive royalties in/an NFT can be traded in (e.g. XRD)
//     permitted_currencies: KeyValueStore<ResourceAddress, ()>,
//     /// Set minimum fixed amounts of royalties for each permitted currency
//     /// this is useful if a creator wants to allow private sales, but still ensure they receive royalties.
//     minimum_royalties: bool,
//     /// Minimum royalty amounts for each currency
//     minimum_royalty_amounts: KeyValueStore<ResourceAddress, Decimal>,
//     // Permissioned dApps - Dapps that you want to allow your NFTs to interact with/be deposited to.
//     limit_dapps: bool,
//     /// A permission list of components an NFT can be transferred to
//     permissioned_dapps: KeyValueStore<ComponentAddress, ()>,
//     /// This is useful because private traders could trade the NFTs without paying royalties, so this closes that loophole.
//     /// However, this can be turned off if the creator wants to allow any trader to trade the NFTs. If a creator wants to allow private sales,
//     /// but still receive royalties - they can set a minimum royalty amount for each currency.
//     limit_buyers: bool,
//     /// A permission list for marketplaces/individual buyers that can trade the NFTs
//     /// This requires that a certain badge is shown by the buyer or marketplace in order to purchase an NFT.
//     permissioned_buyers: KeyValueStore<ResourceAddress, ()>,
//     /// lock royalty configuration: Option can give traders confidence that the royalty percentage/settings will not change.
//     /// There's no method to undo this once set to true. However, right now creators can always take steps to make their
//     /// royalties more relaxed even if locked - i.e. remove mininimum royalties, allow all buyers, etc.
//     royalty_configuration_locked: bool,
// }

// #[derive(ScryptoSbor, NonFungibleData)]
// struct NFT {
//     #[mutable]
//     name: String,
//     #[mutable]
//     description: String,
//     #[mutable]
//     key_image_url: Url,
//     #[mutable]
//     attributes: Vec<HashMap<String, String>>,
// }

#[derive(ScryptoSbor, NonFungibleData)]
struct CreatorKey {
    collection: String,
    authority: String,
    minting_component: ComponentAddress,
    royalty_component: ComponentAddress,
}

// #[derive(ScryptoSbor, ScryptoEvent)]
// struct NewOpenTradeMint {
//     resource_address: ResourceAddress,
//     minting_component: ComponentAddress,
//     royalty_component: ComponentAddress,
// }

// #[derive(ScryptoSbor, ScryptoEvent)]
// struct NewOpenTradeReveal {
//     resource_address: ResourceAddress,
//     minting_component: ComponentAddress,
//     royalty_component: ComponentAddress,
// }

#[blueprint]
mod royal_nft {

    // enable_method_auth! {
    // roles {
    //     admin => updatable_by: [];
    // },
    // methods {
    //     // mint_preview_nft => PUBLIC;
    //     // direct_mint => restrict_to: [admin];
    //     // enable_mint_reveal => restrict_to: [admin];
    //     // upload_metadata => restrict_to: [admin];
    //     // creator_admin => PUBLIC;
    //     // mint_reveal => PUBLIC;
    //     // pay_royalty => PUBLIC;
    //     // transfer_royalty_nft_to_dapp => PUBLIC;
    //     // change_royalty_percentage_fee => restrict_to: [admin];
    //     // lower_maximum_royalty_percentage => restrict_to: [admin];
    //     // restrict_currencies_true => restrict_to: [admin];
    //     // restrict_currencies_false => restrict_to: [admin];
    //     // add_permitted_currency => restrict_to: [admin];
    //     // remove_permitted_currency => restrict_to: [admin];
    //     // enable_minimum_royalties => restrict_to: [admin];
    //     // disable_minimum_royalties => restrict_to: [admin];
    //     // set_minimum_royalty_amount => restrict_to: [admin];
    //     // remove_minimum_royalty_amount => restrict_to: [admin];
    //     // add_permissioned_buyer => restrict_to: [admin];
    //     // remove_permissioned_buyer => restrict_to: [admin];
    //     // limit_dapps_false => restrict_to: [admin];
    //     // limit_dapps_true => restrict_to: [admin];
    //     // add_permissioned_dapp => restrict_to: [admin];
    //     // remove_permissioned_dapp => restrict_to: [admin];
    //     // allow_all_buyers => restrict_to: [admin];
    //     // deny_all_buyers => restrict_to: [admin];
    //     // lock_royalty_configuration => restrict_to: [admin];
    //     // resource_address => PUBLIC;
    // }
    // }

    struct RoyalNfts {
        royalty_component: ComponentAddress,
    }

    impl RoyalNfts {
        pub fn start_minting_nft() -> (ComponentAddress, NonFungibleBucket) {
            let (nft_address_reservation, royalty_component_address) =
                Runtime::allocate_component_address(RoyalNfts::blueprint_id());


                let new_string_local_id = StringNonFungibleLocalId::new("id".to_owned()).unwrap();

            let nft_creator_admin = ResourceBuilder::new_string_non_fungible::<CreatorKey>(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                        metadata_setter => rule!(deny_all);
                        metadata_setter_updater => rule!(deny_all);
                    },
                    init {
                        
                        "type" => "OT Creator Key".to_owned(), locked;
                        "icon_url" => Url::of("https://radixopentrade.netlify.app/img/OT_logo_black.webp"), locked;
                        "royalty_component" => royalty_component_address, locked;
                    }
                })
                .mint_initial_supply([(new_string_local_id, CreatorKey {
                   collection: "thing".to_owned(),
                    authority: "Admin".to_owned(),
                    minting_component: royalty_component_address,
                    royalty_component: royalty_component_address,
                })]);

            // create the rules for the creator of the collection
            let creator_admin_rule = rule!(require_amount(
                dec!(1),
                nft_creator_admin.resource_address()
            ));
            let component_adresss = Self {
                royalty_component: royalty_component_address,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(nft_address_reservation)
            .metadata(metadata! (
                roles {
                    metadata_setter => rule!(deny_all);
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {

                    "description" => "An NFT minting and royalty component.".to_owned(), locked;
                    "dapp_definition" => royalty_component_address, locked;

                }
            ))
            // .roles(roles!(
            //     admin => rule!(require(nft_creator_admin.resource_address()));
            // ))
            .globalize();

            (
                royalty_component_address,
                nft_creator_admin,
               
            )
        }

        // // helper method for tests
        // pub fn resource_address(&self) -> ResourceAddress {
        //     self.nft_manager.address()
        // }

        // pub fn creator_admin(&self) -> ResourceAddress {
        //     self.nft_creator_admin
        // }

        // //admin protect direct mint, returns to creator without any payment required.
        // pub fn direct_mint(
        //     &mut self,
        //     data: Vec<(NonFungibleLocalId, (String, Vec<HashMap<String, String>>))>,
        // ) -> Vec<Bucket> {
        //     let mut return_buckets: Vec<Bucket> = vec![];

        //     for (nft_id, metadata) in data {
        //         let key_image = Url::of(metadata.0.clone());

        //         let nft = NFT {
        //             name: self.mint_id.to_string(),
        //             description: self.description.to_string(),
        //             key_image_url: key_image,
        //             attributes: metadata.1.clone(),
        //         };

        //         let mint = self.nft_manager.mint_non_fungible(&nft_id, nft);

        //         return_buckets.push(mint.into());
        //     }

        //     return_buckets
        // }

        // // if the NFTs being minted will have a buy - then - reveal step
        // pub fn enable_mint_reveal(&mut self) {
        //     self.reveal_step = true;
        // }

        // /// This function allows users to buy a preview of an NFT before it is minted. This acts as a mechanism for random minting.
        // /// Users pay for the mint cost and only a certain limit set by the cap can be minted.
        // /// After the desired number of NFTs have been minted, then the creator can update the metadata on all or some of the NFTs.
        // pub fn mint_preview_nft(
        //     &mut self,
        //     mut payment: Bucket,
        //     mut account: Global<Account>,
        // ) -> Vec<Bucket> {
        //     assert!(
        //         self.reveal_step == true,
        //         "[Mint Reveal] : This NFT doesn't have a reveal step enabled"
        //     );
        //     assert!(
        //         payment.amount() >= self.mint_price,
        //         "[Mint Preview NFT] : Insufficient funds to mint NFT"
        //     );
        //     assert!(
        //         payment.resource_address() == self.mint_currency,
        //         "[Mint Preview NFT] : Incorrect currency to mint NFT"
        //     );

        //     assert!(
        //         self.mint_id < self.collection_cap,
        //         "[Mint Preview NFT] : Collection cap reached"
        //     );

        //     self.mint_payments_vault.put(payment.take(self.mint_price));

        //     let nft = NFT {
        //         name: self.mint_id.to_string(),
        //         description: self.description.to_string(),
        //         key_image_url: Url::of(self.preview_image_url.clone()),
        //         attributes: vec![],
        //     };

        //     let edition = self.mint_id.to_string();
        //     let minted_edition = self
        //         .nft_manager
        //         .mint_non_fungible(&NonFungibleLocalId::Integer(self.mint_id.into()), nft);

        //     self.mint_id += 1;

        //     account.try_deposit_or_abort(minted_edition, None);

        //     let manager_image: Url = self.nft_manager.get_metadata("icon_url").unwrap().unwrap();

        //     let manager_name: String = self.nft_manager.get_metadata("name").unwrap().unwrap();

        //     let receipt_name = format!("{} : {}", manager_name, edition);

        //     let receipt = ResourceBuilder::new_fungible(OwnerRole::None)
        //     .burn_roles(burn_roles! {
        //         burner => rule!(allow_all);
        //         burner_updater => rule!(deny_all);
        //     })
        //         .metadata(metadata! {
        //             roles {
        //                 metadata_locker => rule!(deny_all);
        //                 metadata_locker_updater => rule!(deny_all);
        //                 metadata_setter => rule!(deny_all);
        //                 metadata_setter_updater => rule!(deny_all);
        //             },
        //             init {
        //                 "name" => receipt_name.to_owned(), locked;
        //                 "icon_url" => manager_image, locked;
        //                 "receipt" => "This is a display receipt to show the NFT being transferred to your account in this transaction. You will see this NFT in your wallet after the transaction. You can burn this token if you wish to remove the receipt from your wallet.".to_owned(), locked;
        //             }
        //         })
        //         .mint_initial_supply(1);

        //     // we return any change from the transaction and the preview NFT
        //     vec![payment, receipt.into()]
        // }

        // // this functions allows the creator to upload the metadata for the NFTs to conduct the reveal
        // pub fn upload_metadata(
        //     &mut self,
        //     data: Vec<(NonFungibleLocalId, (String, Vec<HashMap<String, String>>))>,
        // ) {
        //     for (nft_id, metadata) in data {
        //         self.metadata.insert(nft_id, metadata);
        //     }
        // }

        // // this function updates the metadata on an NFT that has already been minted to reveal the collection
        // pub fn mint_reveal(&mut self, nft_proof: Vec<Proof>) {
        //     assert!(
        //         self.reveal_step == true,
        //         "[Mint Reveal] : This NFT doesn't have a reveal step enabled"
        //     );
        //     for proof in nft_proof {
        //         let checked_proof = proof.check(self.nft_manager.address());
        //         let nft_id = checked_proof.as_non_fungible().non_fungible_local_id();
        //         let metadata = self.metadata.get(&nft_id).unwrap();
        //         self.nft_manager.update_non_fungible_data(
        //             &nft_id,
        //             "attributes",
        //             metadata.1.clone(),
        //         );
        //         self.nft_manager.update_non_fungible_data(
        //             &nft_id,
        //             "key_image_url",
        //             Url::of(metadata.0.clone()),
        //         );
        //     }
        // }

        // // This function can be called by trader accounts when an NFT from this collection is purchased.
        // // It takes the payment and an option for an account to send the NFT to.
        // // It uses the royalty percentage set by the creator to determine how much of the payment to take.
        // // We use a keyvaluestore of vaults so that we can store multiple currencies.
        // // We take the NFT as an argument so that we can determine at this point whether we want to enforce advanced royalties settings
        // // where only an account component can own the NFT - in which case we just sent the NFT directly to the input account.
        // // Otherwise, we send the NFT back to the trading account component, where a it could be sent on to another component.
        // pub fn pay_royalty(
        //     &mut self,
        //     nft: ResourceAddress,
        //     mut payment: Bucket,
        //     buyer: ResourceAddress,
        // ) -> Bucket {
        //     let payment_amount = payment.amount();

        //     // check the correct NFT for this royalty component has been passed
        //     assert!(
        //         nft == self.nft_manager.address(),
        //         "[pay_royalty] Incorrect resource passed"
        //     );

        //     if self.royalty_config.limit_buyers {
        //         assert!(
        //             self.royalty_config
        //                 .permissioned_buyers
        //                 .get(&buyer)
        //                 .is_some(),
        //             "This buyer is not permissioned to trade this NFT"
        //         );
        //     }

        //     let currency = payment.resource_address();
        //     let limit_currencies = self.royalty_config.limit_currencies;

        //     if limit_currencies {
        //         assert!(
        //             self.royalty_config
        //                 .permitted_currencies
        //                 .get(&currency)
        //                 .is_some(),
        //             "This currency is not permitted for royalties"
        //         );
        //     }

        //     // send the royalty to the royalty vault

        //     let vault_exists = self.royalty_vaults.get(&currency).is_some();

        //     if !vault_exists {
        //         // check the correct amount has been passed
        //         let royalty = payment.take_advanced(
        //             payment_amount
        //                 .checked_mul(self.royalty_config.royalty_percent)
        //                 .unwrap(),
        //             WithdrawStrategy::Rounded(RoundingMode::ToZero),
        //         );

        //         if limit_currencies {
        //             if self.royalty_config.minimum_royalties {
        //                 let minimum_royalty = self
        //                     .royalty_config
        //                     .minimum_royalty_amounts
        //                     .get(&currency)
        //                     .unwrap();
        //                 assert!(
        //                     royalty.amount() >= minimum_royalty.clone(),
        //                     "Royalty amount is below the minimum required"
        //                 );
        //             }
        //         }

        //         self.royalty_vaults
        //             .insert(currency.clone(), Vault::with_bucket(royalty));
        //     } else {
        //         // check the correct amount has been passed
        //         let royalty = payment.take_advanced(
        //             payment_amount
        //                 .checked_mul(self.royalty_config.royalty_percent)
        //                 .unwrap(),
        //             WithdrawStrategy::Rounded(RoundingMode::ToZero),
        //         );

        //         if limit_currencies {
        //             if self.royalty_config.minimum_royalties {
        //                 let minimum_royalty = self
        //                     .royalty_config
        //                     .minimum_royalty_amounts
        //                     .get(&currency)
        //                     .unwrap();
        //                 assert!(
        //                     royalty.amount() >= minimum_royalty.clone(),
        //                     "Royalty amount is below the minimum required"
        //                 );
        //             }
        //         }
        //         self.royalty_vaults.get_mut(&currency).unwrap().put(royalty);
        //     }

        //     // payment minus royalty returned to the trading account that called this method
        //     payment
        // }

        // /// Possibility to transfer the royalty NFT to a dApp if permissions are set for advanced royalty enforcement - requires the dApp to be permissioned - transfer occurs here.
        // /// If the royalty config allows it, then any dApp can interact with the NFT.
        // /// We allow an optional return of a vector of buckets which should cover most use cases for dApps.
        // ///
        // /// As long as the code remains relatively similar - developers can use this method to have some reactive logic for when their NFTs interact with certain dApps.
        // pub fn transfer_royalty_nft_to_dapp(
        //     &mut self,
        //     nft: Bucket,
        //     dapp: ComponentAddress,
        //     custom_method: String,
        // ) -> Option<Vec<Bucket>> {
        //     if self.royalty_config.limit_dapps {
        //         assert!(
        //             self.royalty_config.permissioned_dapps.get(&dapp).is_some(),
        //             "This dApp has not been permissioned by the collection creator"
        //         );
        //     }

        //     let call_address: Global<AnyComponent> = Global(ObjectStub::new(
        //         ObjectStubHandle::Global(GlobalAddress::from(dapp)),
        //     ));

        //     let manfiest_method: &str = &custom_method;

        //     self.nft_manager.set_depositable(rule!(allow_all));

        //     // send nft to dapp
        //     let optional_returned_buckets =
        //         call_address.call_raw::<Option<Vec<Bucket>>>(manfiest_method, scrypto_args!(nft));

        //     self.nft_manager.set_depositable(rule!(
        //         require_amount(1, self.depositer_admin)
        //             || require(global_caller(self.royalty_component))
        //     ));

        //     optional_returned_buckets
        // }

        // //
        // // These set of methods offer the ability for the creator modify their royalty settings.
        // //

        // /// Only possible if the royalty configuration is not locked
        // /// New percentage fee must be below the maximum set.
        // pub fn change_royalty_percentage_fee(&mut self, new_royalty_percent: Decimal) {
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );

        //     assert!(
        //         new_royalty_percent <= self.royalty_config.maximum_royalty_percent,
        //         "New royalty percentage is greater than maximum allowed"
        //     );

        //     self.royalty_config.royalty_percent = new_royalty_percent;
        // }

        // /// you can always lower the maximum royalty percentage - even if the configuration is locked.
        // pub fn lower_maximum_royalty_percentage(&mut self, new_max_royalty_percent: Decimal) {
        //     assert!(
        //         new_max_royalty_percent >= self.royalty_config.royalty_percent,
        //         "New maximum royalty percentage is less than current royalty percentage"
        //     );

        //     self.royalty_config.maximum_royalty_percent = new_max_royalty_percent;
        // }

        // /// Only possible if the royalty configuration is not locked.
        // /// You can always turn this setting off even if the configuration is locked.
        // pub fn restrict_currencies_true(&mut self) {
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );
        //     self.royalty_config.limit_currencies = true;
        // }

        // pub fn restrict_currencies_false(&mut self) {
        //     self.royalty_config.limit_currencies = false;
        // }

        // // You can only add restricted currencies if the restricted currency setting is turned on.
        // // You can add even if the configuration is locked.
        // pub fn add_permitted_currency(&mut self, currency: ResourceAddress) {
        //     assert!(
        //         self.royalty_config.limit_currencies,
        //         "Restricted currency setting is not turned on"
        //     );
        //     self.royalty_config
        //         .permitted_currencies
        //         .insert(currency, ());
        // }

        // // You can only remove restricted currencies if the restricted currency setting is turned on.
        // // You can't remove currencies if the configuration is locked.
        // pub fn remove_permitted_currency(&mut self, currency: ResourceAddress) {
        //     assert!(
        //         self.royalty_config.limit_currencies,
        //         "Restricted currency setting is not turned on"
        //     );
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );
        //     self.royalty_config.permitted_currencies.remove(&currency);
        // }

        // // You can only set minimum royalty amounts if the restricted currency setting is turned on.

        // // enable minimum royalties

        // pub fn enable_minimum_royalties(&mut self) {
        //     assert!(
        //         self.royalty_config.limit_currencies,
        //         "Restricted currency setting is not turned on"
        //     );
        //     self.royalty_config.minimum_royalties = true;
        // }

        // pub fn disable_minimum_royalties(&mut self) {
        //     self.royalty_config.minimum_royalties = false;
        // }

        // // You can't set minimum amounts if the configuration is locked.
        // pub fn set_minimum_royalty_amount(
        //     &mut self,
        //     currency: ResourceAddress,
        //     minimum_royalty_amount: Decimal,
        // ) {
        //     assert!(
        //         self.royalty_config.limit_currencies,
        //         "Restricted currency setting is not turned on"
        //     );
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );
        //     self.royalty_config
        //         .minimum_royalty_amounts
        //         .insert(currency, minimum_royalty_amount);
        // }

        // // You can only remove minimum royalty amounts if the restricted currency setting is turned on.
        // // You can remove even if the configuration is locked.
        // pub fn remove_minimum_royalty_amount(&mut self, currency: ResourceAddress) {
        //     assert!(
        //         self.royalty_config.limit_currencies,
        //         "Restricted currency setting is not turned on"
        //     );
        //     self.royalty_config
        //         .minimum_royalty_amounts
        //         .remove(&currency);
        // }

        // // Permissioned dapps settings only work with limit dapps enabled.

        // pub fn limit_dapps_true(&mut self) {
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );
        //     self.royalty_config.limit_dapps = true;
        // }

        // pub fn limit_dapps_false(&mut self) {
        //     self.royalty_config.limit_dapps = false;
        // }

        // // You can add even if the configuration is locked.
        // pub fn add_permissioned_dapp(&mut self, dapp: ComponentAddress) {
        //     self.royalty_config.permissioned_dapps.insert(dapp, ());
        // }

        // // You can't remove dapps if the configuration is locked.
        // pub fn remove_permissioned_dapp(&mut self, dapp: ComponentAddress) {
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );
        //     self.royalty_config.permissioned_dapps.remove(&dapp);
        // }

        // // Permissioned buyers settings only work with advanced royalty enforcement settings.
        // // You can always add more permissioned buyers even if the configuration is locked.
        // pub fn add_permissioned_buyer(&mut self, buyer: ResourceAddress) {
        //     self.royalty_config.permissioned_buyers.insert(buyer, ());
        // }

        // // You can't remove buyers if the configuration is locked.
        // pub fn remove_permissioned_buyer(&mut self, buyer: ResourceAddress) {
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );
        //     self.royalty_config.permissioned_buyers.remove(&buyer);
        // }

        // // You can't change to deny_all buyers if the configuration is locked.
        // pub fn deny_all_buyers(&mut self) {
        //     assert!(
        //         !self.royalty_config.royalty_configuration_locked,
        //         "Royalty configuration is locked"
        //     );
        //     self.royalty_config.limit_buyers = true;
        // }

        // // You can allow all buyers even if the configuration is locked
        // pub fn allow_all_buyers(&mut self) {
        //     self.royalty_config.limit_buyers = false;
        // }

        // pub fn lock_royalty_configuration(&mut self) {
        //     self.royalty_config.royalty_configuration_locked = true;
        // }
    }
}
