use scrypto::prelude::*;

#[blueprint]
mod generic_dapp {

    struct Dapp {
        vault: Vault,
    }

    impl Dapp {
        pub fn start_dapp(royalty_nft: Bucket) -> Global<Dapp> {
            Self {
                vault: Vault::with_bucket(royalty_nft),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn withdraw_from_vault(&mut self) -> Bucket {
            self.vault.take_all()
        }
    }
}
