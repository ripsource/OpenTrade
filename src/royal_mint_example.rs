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

#[derive(ScryptoSbor)]
pub enum RoyaltyEnforcementLevel {
    None,
    Partial,
    Full,
}

#[blueprint]
mod royal_rascals {

    struct RoyalRascals {
        rascal_manager: ResourceManager,
        rascal_creator_admin_manager: ResourceManager,
        rascal_creator_admin: ResourceAddress,

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

        /// Royalty enforcement levels can be set at none, partial, or full
        /// None: No royalties are enforced
        /// Partial (reccommended): Royalties are enforced but can be bypassed if a middleman component is used for trading,
        /// however it does allow the NFTs to be used with other components that are not accounts.
        /// Full: Royalties are enforced and cannot be bypassed, this means that the NFTs can only be used with accounts.
        royalty_enforcement_level: RoyaltyEnforcementLevel,

        /// All the royalty payments that have been made for different currencies
        royalty_vaults: KeyValueStore<ResourceAddress, Vault>,

        /// The address of the royalty component (which in this case, is this same component)
        royalty_component: ComponentAddress,
    }

    impl RoyalRascals {
        pub fn start_minting_rascals(
            mint_price: Decimal,
            mint_currency: ResourceAddress,
            collection_cap: u64,
            royalty_percent: Decimal,
            depositer_admin: ResourceAddress,
        ) -> (Global<RoyalRascals>, FungibleBucket) {
            let (rascal_address_reservation, royalty_component_address) =
                Runtime::allocate_component_address(RoyalRascals::blueprint_id());

            assert!(
                royalty_percent <= Decimal::from(1),
                "Royalty percent must be less than 100%"
            );

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
                            "name" => "Royal Rascals", updatable;
                            "royalty_component" => royalty_component_address, updatable;
                        }
                    })
                    .create_with_no_initial_supply();

            let component_adresss = Self {
                rascal_manager,
                royalty_component: royalty_component_address,
                rascal_creator_admin_manager: rascal_creator_admin.resource_manager(),
                rascal_creator_admin: rascal_creator_admin.resource_address(),
                mint_price,
                mint_currency: mint_currency.clone(),
                collection_cap,
                mint_id: 0,
                mint_payments_vault: Vault::new(mint_currency),
                royalty_percent,
                royalty_vaults: KeyValueStore::new(),
                royalty_enforcement_level: RoyaltyEnforcementLevel::Full,
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

            // send nft to account
            account.try_deposit_or_abort(nft, None);

            // payment minus royalty returned
            payment
        }
    }
}
