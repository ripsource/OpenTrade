use crate::opentrader_account::opentrader::OpenTrader;
use scrypto::prelude::*;

/// This blueprint is creates all the trader virtual accounts. It creates virtual badges that are used to authenticate event emitter calls from each trader acccount.
/// It also creates a personal key for each trader account that is used to access their account/make listings, update listings, and cancel listings.

#[derive(ScryptoSbor, NonFungibleData)]
struct TraderKey {}

#[blueprint]
mod openhub {

    struct OpenHub {
        open_trader_key: ResourceManager,
        open_trader_personal_key: ResourceManager,
    }

    impl OpenHub {
        pub fn start_open_hub() -> Global<OpenHub> {
            let (event_address_reservation, event_component_address) =
                Runtime::allocate_component_address(OpenHub::blueprint_id());

            let global_caller_badge_rule = rule!(require(global_caller(event_component_address)));

            let open_trader_key =
                ResourceBuilder::new_ruid_non_fungible::<TraderKey>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            let open_trader_personal_key =
                ResourceBuilder::new_ruid_non_fungible::<TraderKey>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            Self {
                open_trader_key,
                open_trader_personal_key,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(event_address_reservation)
            .globalize()
        }

        pub fn create_open_trader(&self, my_account: Global<Account>) {
            let virtual_badge = self.open_trader_key.mint_ruid_non_fungible(TraderKey {});

            let personal_key = self
                .open_trader_personal_key
                .mint_ruid_non_fungible(TraderKey {});

            let nfgid = NonFungibleGlobalId::new(
                personal_key.resource_address(),
                personal_key.as_non_fungible().non_fungible_local_id(),
            );

            OpenTrader::create_trader(nfgid, my_account, virtual_badge);
        }
    }
}
