use scrypto::prelude::*;
// use crate::royal_mint_example::royal_nft;
// use crate::royal_mint_example::royal_nft::RoyalNFTs;
// use crate::the_mint::royal_nft::RoyalNFTs_start_minting_nft;
// use crate::the_mint::royal_nft::*;
use crate::royal_mint_example::royal_nft::*;

#[derive(ScryptoSbor, ScryptoEvent)]
struct FreshMint {
    mint_component: Global<RoyalNFTs>,
    resource_address: ResourceAddress,
}


#[blueprint]
#[events(FreshMint)]
mod mint_factory {

    struct MintFactory {
        
    }

    impl MintFactory {
        pub fn start_mint_factory() -> (Global<MintFactory>, Bucket) {


            let (address_reservation, component_address) =
                Runtime::allocate_component_address(MintFactory::blueprint_id());

            // let global_caller_badge_rule = rule!(require(global_caller(component_address)));

            let mint_factory_admin: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
              .metadata(metadata!(
                roles {
                  metadata_setter => rule!(deny_all);
                  metadata_setter_updater => rule!(deny_all);
                  metadata_locker => rule!(deny_all);
                  metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "name" => "Mint Factory Admin".to_owned(), locked;
                    "description" => "Mint Factory Admin Badge".to_owned(), locked;
                    "icon_url" => Url::of("https://radixopentrade.netlify.app/img/OT_logo_black.webp"), locked;
                }
              ))
                .divisibility(0)
                .mint_initial_supply(1).into();

            let admin_rule = rule!(require(mint_factory_admin.resource_address()));

(
            Self {
               
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .metadata(metadata! (
                roles {
                    metadata_setter => admin_rule.clone();
                    metadata_setter_updater => admin_rule.clone();
                    metadata_locker => admin_rule.clone();
                    metadata_locker_updater => admin_rule.clone();
                },
                init {
                    "name" => "OT Mint Factory".to_owned(), updatable;
                    "description" => "The mint factory for OT Collections".to_owned(), updatable;
                    "dapp_definition" => component_address, updatable;
                    "icon_url" => Url::of("https://radixopentrade.netlify.app/img/OT_logo_black.webp"), updatable;
                }
            ))
            .with_address(address_reservation)
            .globalize(), mint_factory_admin)



        }

        pub fn create_royal_nft(&mut self,
            name: String,
            description: String,
            icon_url: String,
            preview_image_url: String,
            mint_price: Decimal,
            mint_currency: ResourceAddress,
            collection_cap: u64,
            rules: Vec<bool>,
            depositer_admin: ResourceAddress,
            royalties_enabled: bool,
            royalty_percent: Decimal,
            maximum_royalty_percent: Decimal,
               // These represent some advanced setting that creators can enable to heighten the level of royalty enforcement
            // and use to create new reactive/dynamic features for their NFTs.
            limits: Vec<bool>,
            // 0. limit_buyers: bool,
            // 1. limit_currencies: bool,
            // 2. limit_dapps: bool,
            // 3. limit_private_trade: bool,
            // 4. minimum_royalties: bool,
            permissioned_dapps_input: HashMap<ComponentAddress, ResourceAddress>,
            permissioned_buyers_input: Vec<ResourceAddress>,
            restricted_currencies_input: Vec<ResourceAddress>,
            minimum_royalty_amounts_input: HashMap<ResourceAddress, Decimal>,
        ) -> (Global<RoyalNFTs>, NonFungibleBucket, ResourceAddress) {


            let fresh_mint: (Global<RoyalNFTs>, NonFungibleBucket, ResourceAddress) = RoyalNFTs::start_minting_nft(
                name,
                description,
                icon_url,
                preview_image_url,
                mint_price,
                mint_currency,
                collection_cap,
                rules,
                depositer_admin,
                royalties_enabled,
                royalty_percent,
                maximum_royalty_percent,
                limits,
                permissioned_dapps_input,
                permissioned_buyers_input,
                restricted_currencies_input,
                minimum_royalty_amounts_input
            );

            Runtime::emit_event(FreshMint {
                mint_component: fresh_mint.0,
                resource_address: fresh_mint.2,
            });

            fresh_mint
        }
    
    }

}