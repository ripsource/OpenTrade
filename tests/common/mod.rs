//! Contains common library functions for the test code.
use scrypto_test::prelude::*;

use ledger_simulator::{DefaultLedgerSimulator, LedgerSimulatorBuilder};
// use escrow::token_quantity::TokenQuantity;
// use radix_engine::transaction::{BalanceChange, CommitResult};
// use radix_engine_common::ManifestSbor;
use scrypto::prelude::*;
use scrypto_test::{
    prelude::{BalanceChange, CommitResult},
    *,
};
// use transaction::prelude::*;

#[derive(ScryptoSbor, ManifestSbor, NonFungibleData)]
pub struct NfData {}

#[derive(Clone)]
pub struct User {
    pub pubkey: Secp256k1PublicKey,
    pub account: ComponentAddress,
    pub nfgid: NonFungibleGlobalId,
    pub display_name: Option<String>,
}

impl User {
    const EMPTY: &'static str = "";
    pub fn display_name(&self) -> &str {
        if let Some(str) = &self.display_name {
            return &str;
        }
        Self::EMPTY
    }
}

pub fn make_user(test_runner: &mut DefaultLedgerSimulator, display_name: Option<&str>) -> User {
    let (user_pubk, _, user_account) = test_runner.new_allocated_account();

    User {
        nfgid: NonFungibleGlobalId::from_public_key(&user_pubk),
        pubkey: user_pubk,
        account: user_account,
        display_name: display_name.map(|v| v.to_string()),
    }
}

pub fn balance_change_amount(
    commit_result: &CommitResult,
    vaults: Vec<NodeId>,
    resource: ResourceAddress,
) -> Decimal {
    for (_, (vault_id, (vault_resource, delta))) in
        commit_result.vault_balance_changes().iter().enumerate()
    {
        if resource == *vault_resource && vaults.contains(vault_id) {
            match delta {
                BalanceChange::Fungible(d) => return *d,
                BalanceChange::NonFungible { added, removed } => {
                    return Decimal::from(added.len() as i64 - removed.len() as i64)
                }
            }
        }
    }
    return Decimal::ZERO;
}

pub fn balance_change_nflids(
    commit_result: &CommitResult,
    vaults: Vec<NodeId>,
    resource: ResourceAddress,
) -> (BTreeSet<NonFungibleLocalId>, BTreeSet<NonFungibleLocalId>) {
    for (_, (vault_id, (vault_resource, delta))) in
        commit_result.vault_balance_changes().iter().enumerate()
    {
        if resource == *vault_resource && vaults.contains(vault_id) {
            match delta {
                BalanceChange::NonFungible { added, removed } => {
                    return (added.clone(), removed.clone())
                }
                BalanceChange::Fungible(_) => {}
            }
        }
    }
    return (BTreeSet::new(), BTreeSet::new());
}

/// Creates the test runner, a user, and publishes the package under
/// test.
pub fn setup_for_test() -> (DefaultLedgerSimulator, User, PackageAddress) {
    let mut test_runner = LedgerSimulatorBuilder::new()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .without_kernel_trace()
        .build();
    let alice = make_user(&mut test_runner, Some(&"Alice".to_owned()));
    let package_address = test_runner.compile_and_publish(this_package!());

    (test_runner, alice, package_address)
}

/// Retrieves all non-fungible local ids of a given resource held by
/// an account. For non-fungibles it's like
/// TestRunner::get_component_balance except better.
pub fn get_component_nflids(
    test_runner: &mut DefaultLedgerSimulator,
    account: ComponentAddress,
    resource: ResourceAddress,
) -> BTreeSet<NonFungibleLocalId> {
    let mut nflids: BTreeSet<NonFungibleLocalId> = BTreeSet::new();
    let vaults = test_runner.get_component_vaults(account, resource);
    for vault in vaults {
        if let Some((_, nfs)) = test_runner.inspect_non_fungible_vault(vault) {
            nflids.extend(nfs);
        }
    }
    nflids
}

use std::ops::Range;

/// Converts a vector of u64 into a vector of NonFungibleLocalId
pub fn to_nflids(ints: Range<u64>) -> IndexSet<NonFungibleLocalId> {
    let ints: Vec<u64> = ints.collect();
    ints.into_iter()
        .map(|n| NonFungibleLocalId::Integer(n.into()))
        .collect()
}

/// Creates an NFT resource with integer-based local ids. Local ids
/// will start on `base` and count upwards until there are `amount`
/// NFTs in the resource. All NFTs will be given to `owner_account`.
pub fn create_nft_resource(
    test_runner: &mut DefaultLedgerSimulator,
    owner: &User,
    base: u64,
    amount: u64,
    badge: Option<&ResourceAddress>,
) -> ResourceAddress {
    let owner_nfgid = NonFungibleGlobalId::from_public_key(&owner.pubkey);

    let roles = NonFungibleResourceRoles {
        mint_roles: mint_roles!(
            minter => if let Some(badge_resaddr) = badge {
                rule!(require(*badge_resaddr))
            } else {
                rule!(allow_all)
            };
            minter_updater => rule!(allow_all);
        ),
        withdraw_roles: withdraw_roles!(
            withdrawer => rule!(allow_all);
            withdrawer_updater => rule!(deny_all);
        ),
        deposit_roles: deposit_roles!(
            depositor => rule!(allow_all);
            depositor_updater => rule!(deny_all);
        ),
        burn_roles: None,
        freeze_roles: None,
        recall_roles: None,
        non_fungible_data_update_roles: None,
    };

    // We're just faking the simplest None we can get away with here
    // (because faking it with the usual None::<String> doesn't work
    // in this case)
    let empty_supply: Option<Vec<(NonFungibleLocalId, NfData)>> = None;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_non_fungible_resource(
            OwnerRole::Fixed(rule!(require(owner_nfgid.clone()))),
            NonFungibleIdType::Integer,
            true,
            roles,
            metadata!(),
            empty_supply,
        )
        .deposit_batch(owner.account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![owner_nfgid.clone()]);

    receipt.expect_commit_success();
    let resaddr = receipt.expect_commit(true).new_resource_addresses()[0];

    let mut minted = 0;
    const BATCH_SIZE: u64 = 100;

    // We mint in batches because there is a max-substates-write limit
    // that we might hit otherwise when making lots of NFTs.
    while minted < amount {
        let mut to_mint = amount - minted;
        if to_mint > BATCH_SIZE {
            to_mint = BATCH_SIZE;
        }

        let mut builder = ManifestBuilder::new();
        if let Some(badge_resaddr) = badge {
            builder =
                builder.create_proof_from_account_of_amount(owner.account, *badge_resaddr, dec!(1));
        }
        let manifest = builder
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resaddr,
                (minted..minted + to_mint)
                    .map(|n| (NonFungibleLocalId::Integer((base + n).into()), NfData {}))
                    .collect::<HashMap<NonFungibleLocalId, NfData>>(),
            )
            .deposit_batch(owner.account)
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![owner_nfgid.clone()]);

        receipt.expect_commit_success();
        minted += to_mint;
    }

    resaddr
}
