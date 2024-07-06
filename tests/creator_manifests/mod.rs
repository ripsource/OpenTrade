use scrypto::prelude::*;
use scrypto_test::prelude::*;

use crate::common::*;

pub fn enable_mint_reveal(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_admin: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_admin, dec!(1)),
        )
        .call_method(component, "enable_mint_reveal", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn mint_royalty_nft(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(user.account, "withdraw", manifest_args!(XRD, dec!(100)))
        .take_all_from_worktop(XRD, "payment")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component,
                "mint_preview_nft",
                manifest_args!(lookup.bucket("payment"), user.account),
            )
        })
        .call_method(
            user.account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }
}

pub fn nft_address(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
) -> ResourceAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "resource_address", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true).output(1)
}

pub fn create_global_id(nft_address: ResourceAddress, number: u64) -> NonFungibleGlobalId {
    let local_id: NonFungibleLocalId = NonFungibleLocalId::integer(number.into());
    NonFungibleGlobalId::new(nft_address.clone(), local_id.clone())
}

pub fn change_burn_rule(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    new_rule: AccessRule,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .pop_from_auth_zone("creator_proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component,
                "change_burn_rule",
                manifest_args!(new_rule, lookup.proof("creator_proof")),
            )
        })
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn lock_burn_rule(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .pop_from_auth_zone("creator_proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component,
                "lock_burn_rule",
                manifest_args!(lookup.proof("creator_proof")),
            )
        })
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn change_metadata_updatable_rule(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    new_rule: AccessRule,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .pop_from_auth_zone("creator_proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component,
                "change_metadata_updatable_rule",
                manifest_args!(new_rule, lookup.proof("creator_proof")),
            )
        })
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn lock_metadata_updatable_rule(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .pop_from_auth_zone("creator_proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component,
                "lock_metadata_updatable_rule",
                manifest_args!(lookup.proof("creator_proof")),
            )
        })
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn change_mint_rule(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    new_rule: AccessRule,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .pop_from_auth_zone("creator_proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component,
                "change_mint_rule",
                manifest_args!(new_rule, lookup.proof("creator_proof")),
            )
        })
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn lock_mint_rule(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .pop_from_auth_zone("creator_proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component,
                "lock_mint_rule",
                manifest_args!(lookup.proof("creator_proof")),
            )
        })
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn change_royalty_percentage_fee(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    new_fee: Decimal,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(
            component,
            "change_royalty_percentage_fee",
            manifest_args!(new_fee),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn lower_maximum_royalty_percentage_fee(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    new_fee: Decimal,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(
            component,
            "lower_maximum_royalty_percentage",
            manifest_args!(new_fee),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn restrict_currencies_false(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "restrict_currencies_false", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn restrict_currencies_true(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "restrict_currencies_true", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn add_permitted_currency(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    add_currency: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(
            component,
            "add_permitted_currency",
            manifest_args!(add_currency),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn remove_permitted_currency(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    remove_currency: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(
            component,
            "remove_permitted_currency",
            manifest_args!(remove_currency),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn set_minimum_royalty_amount(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    currency: ResourceAddress,
    new_minimum: Decimal,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(
            component,
            "set_minimum_royalty_amount",
            manifest_args!(currency, new_minimum),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn remove_minimum_royalty_amount(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    currency: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(
            component,
            "remove_minimum_royalty_amount",
            manifest_args!(currency),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn add_permissioned_dapp(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    dapp: ComponentAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "add_permissioned_dapp", manifest_args!(dapp))
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn remove_permissioned_dapp(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    dapp: ComponentAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "remove_permissioned_dapp", manifest_args!(dapp))
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn add_permissioned_buyer(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    buyer: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "add_permissioned_buyer", manifest_args!(buyer))
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn remove_permissioned_buyer(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
    buyer: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(
            component,
            "remove_permissioned_buyer",
            manifest_args!(buyer),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn deny_all_buyers(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "deny_all_buyers", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn allow_all_buyers(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "allow_all_buyers", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}

pub fn lock_royalty_configuration(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
    creator_key: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_amount",
            manifest_args!(creator_key, dec!(1)),
        )
        .call_method(component, "lock_royalty_configuration", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    receipt.expect_commit(true);
}
