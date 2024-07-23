use crate::open_trade_event::event;
use crate::open_trade_event::event::Event;
use crate::open_trader_account::opentrader::OpenTrader;
use scrypto::prelude::*;

// This blueprint creates all the open trader accounts. It creates emitter badges that are used to authenticate event emitters from each trader acccount and allows
// traders to buy and sell Royalty NFTs. It also creates a personal key for each trader account that is used to access their account/make listings, update listings,
// and cancel listings.

#[derive(ScryptoSbor, NonFungibleData)]
struct TraderKey {
    name: String,
}

#[blueprint]
mod openhub {

    struct OpenHub {
        /// The badge that is stored and locked in a trader account to authenticate event emitters
        emitter_trader_badge: ResourceManager,
        /// The personal user badge that a user holds and uses to authenticate methods on their trading account
        open_trader_account_badge: ResourceManager,
        /// The badge that is used to allow trader accounts to hold and trade Royalty NFTs
        royal_nft_depositer_badge: ResourceManager,
        /// Event emitter component
        event_manager: Global<event::Event>,
        /// Hub Component Address
        component_address: ComponentAddress,
        /// AccountLocker for all traders
        account_locker: Global<AccountLocker>,
    }

    impl OpenHub {
        /// Instantiation of the open hub component creates the resource managers of all the key badges used in the system
        /// which are minted when a user creates a trading account for themselves.
        pub fn start_open_hub() -> Global<OpenHub> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(OpenHub::blueprint_id());

            let global_caller_badge_rule = rule!(require(global_caller(component_address)));

            let emitter_trader_badge =
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

            let event_manager = Event::create_event_listener(emitter_trader_badge.address());

            let locker_badge_rule = rule!(require(emitter_trader_badge.address()));

            let locker = Blueprint::<AccountLocker>::instantiate(
                OwnerRole::None,   // owner
                locker_badge_rule, // storer
                rule!(deny_all),   // storer_updater
                rule!(deny_all),   // recoverer
                rule!(deny_all),   // recoverer_updater
                None,              // address_reservation
            );

            Self {
                emitter_trader_badge,
                open_trader_account_badge,
                royal_nft_depositer_badge,
                event_manager,
                component_address,
                account_locker: locker,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .metadata(metadata! (
                roles {
                    metadata_setter => rule!(deny_all);
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "name" => "OpenTrade".to_owned(), locked;
                    "description" => "OpenTrade Hub".to_owned(), locked;
                    "dapp_definition" => component_address, locked;
                    "icon_url" => Url::of("https://radixopentrade.netlify.app/img/OT_logo_black.webp"), locked;
                }
            ))
            .with_address(address_reservation)
            .globalize()
        }

       

        /// Creates a new open trader account with a emitter badge, personal key, and a badge to hold and trade Royalty NFTs
        pub fn create_open_trader(
            &self,
            my_account: Global<Account>,
        ) -> (NonFungibleGlobalId, Bucket) {
            let emitter_badge = self.emitter_trader_badge.mint_ruid_non_fungible(TraderKey {
                name: "emitter".to_string(),
            });

            let personal_trading_account_badge = self
                .open_trader_account_badge
                .mint_ruid_non_fungible(TraderKey {
                    name: "trader_account".to_string(),
                });

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
                emitter_badge,
                depositer_permission_badge,
                self.event_manager,
                self.component_address,
                self.account_locker.clone(),
            );

            // return the personal trading account badge (and the nfgid of the account for testing purposes)
            (nfgid, personal_trading_account_badge)
        }

        pub fn fetch_virt_badge(&mut self) -> ResourceAddress {
            self.emitter_trader_badge.address()
        }

        pub fn fetch_royal_nft_depositer_badge(&mut self) -> ResourceAddress {
            self.royal_nft_depositer_badge.address()
        }
    }
}
