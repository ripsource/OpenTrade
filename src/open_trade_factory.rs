use crate::open_trader_account::opentrader::OpenTrader;
use scrypto::prelude::*;

// This blueprint creates all the open trader accounts. It creates virtual badges that are used to authenticate event emitters from each trader acccount and allows
// traders to buy and sell Royalty NFTs. It also creates a personal key for each trader account that is used to access their account/make listings, update listings,
// and cancel listings.

#[derive(ScryptoSbor, NonFungibleData)]
struct TraderKey {}

#[blueprint]
mod openhub {

    struct OpenHub {
        /// The badge that is stored and locked in a trader account to authenticate event emitters
        virtual_trader_badge: ResourceManager,
        /// The personal user badge that a user holds and uses to authenticate methods on their trading account
        open_trader_account_badge: ResourceManager,
        /// The badge that is used to allow trader accounts to hold and trade Royalty NFTs
        royal_nft_depositer_badge: ResourceManager,
    }

    impl OpenHub {
        /// Instantiation of the open hub component creates the resource managers of all the key badges used in the system
        /// which are minted when a user creates a trading account for themselves.
        pub fn start_open_hub() -> Global<OpenHub> {
            let (event_address_reservation, event_component_address) =
                Runtime::allocate_component_address(OpenHub::blueprint_id());

            let global_caller_badge_rule = rule!(require(global_caller(event_component_address)));

            let virtual_trader_badge =
                ResourceBuilder::new_ruid_non_fungible::<TraderKey>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            let open_trader_account_badge =
                ResourceBuilder::new_ruid_non_fungible::<TraderKey>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            let royal_nft_depositer_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_roles(mint_roles! {
                    minter => global_caller_badge_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .divisibility(0)
                .create_with_no_initial_supply();

            Self {
                virtual_trader_badge,
                open_trader_account_badge,
                royal_nft_depositer_badge,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(event_address_reservation)
            .globalize()
        }

        /// Creates a new open trader account with a virtual badge, personal key, and a badge to hold and trade Royalty NFTs
        pub fn create_open_trader(
            &self,
            my_account: Global<Account>,
        ) -> (NonFungibleGlobalId, Bucket) {
            let virtual_badge = self
                .virtual_trader_badge
                .mint_ruid_non_fungible(TraderKey {});

            let personal_trading_account_badge = self
                .open_trader_account_badge
                .mint_ruid_non_fungible(TraderKey {});

            let nfgid = NonFungibleGlobalId::new(
                personal_trading_account_badge.resource_address(),
                personal_trading_account_badge
                    .as_non_fungible()
                    .non_fungible_local_id(),
            );

            let depositer_permission_badge = self.royal_nft_depositer_badge.mint(1);

            // Instatiation of a trading account via the open_trader_account blueprint, passing in badges that will be locked in the accounts.
            OpenTrader::create_trader(
                nfgid.clone(),
                my_account,
                virtual_badge,
                depositer_permission_badge,
            );

            // return the personal trading account badge (and the nfgid of the account for testing purposes)
            (nfgid, personal_trading_account_badge)
        }

        pub fn fetch_virt_badge(&mut self) -> ResourceAddress {
            self.virtual_trader_badge.address()
        }

        pub fn fetch_royal_nft_depositer_badge(&mut self) -> ResourceAddress {
            self.royal_nft_depositer_badge.address()
        }
    }
}
