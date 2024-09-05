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
    description: String,
    key_image_url: Url,
    #[mutable]
    hub: Option<ComponentAddress>,
}

#[derive(ScryptoSbor, NonFungibleData)]
struct EmitterKey {
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
        /// Created accounts
        registered_accounts: KeyValueStore<ComponentAddress, ComponentAddress>,
    }

    impl OpenHub {
        /// Instantiation of the open hub component creates the resource managers of all the key badges used in the system
        /// which are minted when a user creates a trading account for themselves.
        pub fn start_open_hub() -> (Global<OpenHub>, Bucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(OpenHub::blueprint_id());

            let global_caller_badge_rule = rule!(require(global_caller(component_address)));

            let open_hub_admin: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
              .metadata(metadata!(
                roles {
                  metadata_setter => rule!(deny_all);
                  metadata_setter_updater => rule!(deny_all);
                  metadata_locker => rule!(deny_all);
                  metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "name" => "OpenTrade Admin".to_owned(), locked;
                    "description" => "OpenTrade Admin Badge".to_owned(), locked;
                    "icon_url" => Url::of("https://radixopentrade.netlify.app/img/OT_logo_black.webp"), locked;
                }
              ))
                .divisibility(0)
                .mint_initial_supply(1).into();

            let admin_rule = rule!(require(open_hub_admin.resource_address()));

            let emitter_trader_badge =
                ResourceBuilder::new_ruid_non_fungible::<EmitterKey>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => admin_rule.clone();
                    })
                    .create_with_no_initial_supply();

            let open_trader_account_badge =
                ResourceBuilder::new_ruid_non_fungible::<TraderKey>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => global_caller_badge_rule.clone();
                        minter_updater => admin_rule.clone();
                    })
                    .withdraw_roles(withdraw_roles! {
                        withdrawer => rule!(deny_all);
                        withdrawer_updater => admin_rule.clone();
                    })
                    .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                        non_fungible_data_updater => global_caller_badge_rule.clone();
                        non_fungible_data_updater_updater => admin_rule.clone();
                    })
                    .create_with_no_initial_supply();

            let royal_nft_depositer_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_roles(mint_roles! {
                    minter => global_caller_badge_rule.clone();
                    minter_updater => admin_rule.clone();
                })
                .divisibility(0)
                .create_with_no_initial_supply();

            let event_manager = Event::create_event_listener(emitter_trader_badge.address());

            let locker_badge_rule = rule!(require(emitter_trader_badge.address()));

            let locker = Blueprint::<AccountLocker>::instantiate(
                OwnerRole::None,    // owner
                locker_badge_rule,  // storer
                admin_rule.clone(), // storer_updater
                rule!(deny_all),    // recoverer
                rule!(deny_all),    // recoverer_updater
                None,               // address_reservation
            );

            (Self {
                emitter_trader_badge,
                open_trader_account_badge,
                royal_nft_depositer_badge,
                event_manager,
                component_address,
                account_locker: locker,
                registered_accounts: KeyValueStore::new(),
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
                    "name" => "OpenTrade".to_owned(), updatable;
                    "description" => "OpenTrade Hub".to_owned(), updatable;
                    "dapp_definition" => component_address, updatable;
                    "icon_url" => Url::of("https://radixopentrade.netlify.app/img/OT_logo_black.webp"), updatable;
                }
            ))
            .with_address(address_reservation)
            .globalize(), open_hub_admin)
        }

        /// Creates a new open trader account with a emitter badge, personal key, and a badge to hold and trade Royalty NFTs
        pub fn create_open_trader(
            &self,
            my_account: Global<Account>,
        ) -> (NonFungibleGlobalId, Bucket) {
            {
                // Getting the owner role of the account.
                let owner_role = my_account.get_owner_role();

                // Assert against it.
                Runtime::assert_access_rule(owner_role.rule);

                // Assertion passed - the caller is the owner of the account.
            }

            if self
                .registered_accounts
                .get(&my_account.address())
                .is_some()
            {
                panic!("Account already has created an OT Trading Hub - check your wallet for your hub key.");
            }

            let emitter_badge = self
                .emitter_trader_badge
                .mint_ruid_non_fungible(EmitterKey {
                    name: "emitter".to_string(),
                });

            let hub_address = None as Option<ComponentAddress>;

            let personal_trading_account_badge = self
                .open_trader_account_badge
                .mint_ruid_non_fungible(TraderKey {
                    name: "OT Hub Key".to_string(),
                    description: "Your hub for listing and managing your NFTs across marketplaces and with other users.".to_string(),
                    key_image_url: Url::of("https://radixopentrade.netlify.app/img/OT_logo_black.webp"),
                    hub: hub_address,
                });

            let nfgid = NonFungibleGlobalId::new(
                personal_trading_account_badge.resource_address(),
                personal_trading_account_badge
                    .as_non_fungible()
                    .non_fungible_local_id(),
            );

            let depositer_permission_badge = self.royal_nft_depositer_badge.mint(1);

            // Instatiation of a trading account via the open_trader_account blueprint, passing in badges that will be locked in the accounts.
            let new_hub_component = OpenTrader::create_trader(
                nfgid.clone(),
                my_account,
                emitter_badge,
                depositer_permission_badge,
                self.event_manager,
                self.component_address,
                self.account_locker.clone(),
            );

            let hub_component_address = new_hub_component.address();

            self.open_trader_account_badge.update_non_fungible_data(
                nfgid.clone().local_id(),
                "hub",
                Some(hub_component_address.clone()),
            );

            self.registered_accounts
                .insert(my_account.clone().address(), hub_component_address);

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
