use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct Rascal {
    name: String,
    description: String,
    key_image_url: Url,
    attributes: Vec<HashMap<String, String>>,
    royalty_component: ComponentAddress,
}

#[blueprint]
mod royal_rascals {

    struct RoyalRascals {
        rascal_manager: ResourceManager,
        rascal_component: ComponentAddress,
        rascal_admin_manager: ResourceManager,
        rascal_admin: ResourceAddress,
        mint_price: Decimal,
        mint_currency: ResourceAddress,
        collection_cap: u64,
        mint_id: u64,
        mint_payments_vault: Vault,
        royalty_percent: Decimal,
        royally_listed: KeyValueStore<NonFungibleLocalId, ResourceAddress>,
        royalty_vaults: KeyValueStore<ResourceAddress, Vault>,
    }

    impl RoyalRascals {
        pub fn start_minting_rascals(
            mint_price: Decimal,
            mint_currency: ResourceAddress,
            collection_cap: u64,
            royalty_percent: Decimal,
            depositer_admin: ResourceAddress,
        ) -> (Global<RoyalRascals>, FungibleBucket) {
            let (rascal_address_reservation, rascal_component_address) =
                Runtime::allocate_component_address(RoyalRascals::blueprint_id());

            assert!(
                royalty_percent <= Decimal::from(1),
                "Royalty percent must be less than 100%"
            );

            let rascal_admin = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .mint_initial_supply(1);

            let rascal_rule = rule!(require(rascal_admin.resource_address()));

            let global_caller_badge_rule = rule!(require(global_caller(rascal_component_address)));

            let depositer_admin_rule = rule!(
                require_amount(1, depositer_admin)
                    || require(global_caller(rascal_component_address))
            );

            let rascal_manager =
                ResourceBuilder::new_integer_non_fungible::<Rascal>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => rascal_rule.clone();
                    })
                    .burn_roles(burn_roles! {
                        burner => rascal_rule.clone();
                        burner_updater => rascal_rule.clone();
                    })
                    .deposit_roles(deposit_roles! {
                        depositor => depositer_admin_rule;
                        depositor_updater => global_caller_badge_rule.clone();
                    })
                    .recall_roles(recall_roles! {
                        recaller => global_caller_badge_rule.clone();
                        recaller_updater => rule!(deny_all);
                    })
                    .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                        non_fungible_data_updater => rascal_rule.clone();
                        non_fungible_data_updater_updater => rascal_rule.clone();
                    })
                    .metadata(metadata! {
                        roles {
                            metadata_locker => rule!(allow_all);
                            metadata_locker_updater => rule!(allow_all);
                            metadata_setter => OWNER;
                            metadata_setter_updater => rule!(deny_all);
                        },
                        init {
                            "name" => "Royal Rascals", updatable;
                            "royalty_component" => rascal_component_address, updatable;
                        }
                    })
                    .create_with_no_initial_supply();

            let component_adresss = Self {
                rascal_manager,
                rascal_component: rascal_component_address,
                rascal_admin_manager: rascal_admin.resource_manager(),
                rascal_admin: rascal_admin.resource_address(),
                mint_price,
                mint_currency: mint_currency.clone(),
                collection_cap,
                mint_id: 0,
                mint_payments_vault: Vault::new(mint_currency),
                royalty_percent,
                royally_listed: KeyValueStore::new(),
                royalty_vaults: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(rascal_address_reservation)
            .globalize();

            (component_adresss, rascal_admin)
        }

        pub fn resource_address(&self) -> ResourceAddress {
            self.rascal_manager.address()
        }

        pub fn mint_preview_nft(
            &mut self,
            mut payment: Bucket,
            mut account: Global<Account>,
        ) -> Vec<Bucket> {
            {
                let owner_role = account.get_owner_role();
                Runtime::assert_access_rule(owner_role.rule);
            }
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

            {
                // Getting the owner role of the account.
                let owner_role = account.get_owner_role();

                // Assert against it.
                Runtime::assert_access_rule(owner_role.rule);

                // Assertion passed - the caller is the owner of the account.
            }

            self.mint_payments_vault.put(payment.take(self.mint_price));

            let rascal = Rascal {
                name: "Rascal".to_string(),
                description: "A mischievous little rascal".to_string(),
                key_image_url: Url::of("https://rascal.com/placeholder.png"),
                attributes: vec![],
                royalty_component: self.rascal_component,
            };

            let minted_edition = self
                .rascal_manager
                .mint_non_fungible(&NonFungibleLocalId::Integer(self.mint_id.into()), rascal);

            account.try_deposit_or_abort(minted_edition, None);

            self.mint_id += 1;

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
            rascal_admin: Proof,
        ) {
            let checked_admin = rascal_admin.check(self.rascal_admin);

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

        pub fn register_listing(&mut self, nft: Bucket, currency: ResourceAddress) -> Bucket {
            // check the correct resource has been passed
            assert!(
                nft.resource_address() == self.rascal_manager.address(),
                "[royal_list] Incorrect resource passed"
            );

            assert!(
                nft.amount() == dec!(1),
                "[royal_list] Only one NFT can be listed at a time"
            );

            // get the local id of the NFT
            let id = nft.as_non_fungible().non_fungible_local_id();

            // Store the royalty required and currency in the royally_listed store
            self.royally_listed.insert(id.clone(), currency);

            // // allow the NFT to be withdrawn and recall it for the user and store it in the warehouse
            // self.rascal_manager.set_depositable(rule!(allow_all));

            // // Send it to the warehoure
            // warehouse_address.call_raw::<()>("deposit", scrypto_args!(nft));

            // // relock the NFT's movement
            // self.rascal_manager.set_depositable(rule!(deny_all));
            nft
        }

        pub fn pay_royalty(
            &mut self,
            nft: Bucket,
            mut payment: Bucket,
            mut account: Global<Account>,
        ) -> Bucket {
            let payment_amount = payment.amount();

            // check the correct proof has been passed
            assert!(
                nft.resource_address() == self.rascal_manager.address(),
                "[pay_royalty] Incorrect resource passed"
            );

            // get the local id of the NFT
            let nft_id = nft.as_non_fungible().non_fungible_local_id();

            let currency = self
                .royally_listed
                .get(&nft_id)
                .expect("[pay_royalty] NFT not found in royally_listed store")
                .clone();

            // check the correct resource has been passed
            assert!(
                payment.resource_address() == currency,
                "[pay_royalty] Incorrect currency passed"
            );

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

            // remove the NFT from the royally_listed store
            self.royally_listed.remove(&nft_id);

            // we can now allow movement of the NFT
            self.rascal_manager.set_depositable(rule!(allow_all));

            // send nft to account
            account.try_deposit_or_abort(nft, None);

            // relock the NFT's movement
            self.rascal_manager.set_depositable(rule!(deny_all));

            // payment minus royalty returned
            payment
        }
    }
}
