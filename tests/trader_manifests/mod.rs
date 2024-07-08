use scrypto::prelude::*;
use scrypto_test::prelude::*;

use crate::common::*;

pub fn trader_auth_key(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    trader_component: ComponentAddress,
) -> (ResourceAddress, NonFungibleLocalId) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(trader_component, "fetch_auth_key", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    let (trader_auth_resource, trader_auth_local): (ResourceAddress, NonFungibleLocalId) =
        receipt.expect_commit(true).output(1);

    (trader_auth_resource, trader_auth_local)
}

pub fn list_royalty_nft(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    trader_component: ComponentAddress,
    trader_key_resource: ResourceAddress,
    trader_key_local: NonFungibleLocalId,
    nft_address: ResourceAddress,
    nft_local_id: NonFungibleLocalId,
    price: Decimal,
    currency: Option<ResourceAddress>,
    auth_buyers: Vec<ResourceAddress>,
) {
    let sell_currency: ResourceAddress;

    if currency.is_some() {
        sell_currency = currency.unwrap();
    } else {
        sell_currency = XRD;
    }

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_non_fungibles",
            manifest_args!(trader_key_resource, indexset![trader_key_local.clone()]),
        )
        .call_method(
            user.account,
            "withdraw_non_fungibles",
            manifest_args!(nft_address, indexset![nft_local_id.clone()]),
        )
        .take_all_from_worktop(nft_address, "listing")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                trader_component,
                "royal_list",
                manifest_args!(lookup.bucket("listing"), price, sell_currency, auth_buyers,),
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

pub fn purchase_royalty_nft(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    marketplace_component: ComponentAddress,
    trader_component: ComponentAddress,
    nfgid: NonFungibleGlobalId,
    payment: Decimal,
    currency: Option<ResourceAddress>,
) {
    let buy_currency: ResourceAddress;

    if currency.is_some() {
        buy_currency = currency.unwrap();
    } else {
        buy_currency = XRD;
    }

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "withdraw",
            manifest_args!(buy_currency, payment),
        )
        .take_all_from_worktop(buy_currency, "payment")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                marketplace_component,
                "purchase_royal_listing",
                manifest_args!(
                    nfgid,
                    lookup.bucket("payment"),
                    trader_component,
                    user.account,
                ),
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

pub fn cancel_royal_listing(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    trader_component: ComponentAddress,
    trader_key_resource: ResourceAddress,
    trader_key_local: NonFungibleLocalId,
    nfgid: NonFungibleGlobalId,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_non_fungibles",
            manifest_args!(trader_key_resource, indexset![trader_key_local.clone()]),
        )
        .call_method(
            trader_component,
            "cancel_royal_listing",
            manifest_args!(nfgid),
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

pub fn same_owner_royal_transfer(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    trader_component: ComponentAddress,
    nft_address: ResourceAddress,
    nft_local_id: NonFungibleLocalId,
    other_account: ComponentAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "withdraw_non_fungibles",
            manifest_args!(nft_address, indexset![nft_local_id.clone()]),
        )
        .take_all_from_worktop(nft_address, "transfer")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                trader_component,
                "royal_transfer",
                manifest_args!(lookup.bucket("transfer"), other_account),
            )
        })
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

pub fn transfer_royal_nft_to_component(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    trader_component: ComponentAddress,
    nft_address: ResourceAddress,
    nft_local_id: NonFungibleLocalId,
    other_component: ComponentAddress,
    custom_method: String,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "withdraw_non_fungibles",
            manifest_args!(nft_address, indexset![nft_local_id.clone()]),
        )
        .take_all_from_worktop(nft_address, "transfer")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                trader_component,
                "transfer_royal_nft_to_component",
                manifest_args!(lookup.bucket("transfer"), other_component, custom_method),
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

pub fn list(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    trader_component: ComponentAddress,
    trader_key_resource: ResourceAddress,
    trader_key_local: NonFungibleLocalId,
    nft_address: ResourceAddress,
    nft_local_id: NonFungibleLocalId,
    price: Decimal,
    currency: Option<ResourceAddress>,
    auth_buyers: Vec<ResourceAddress>,
) {
    let sell_currency: ResourceAddress;

    if currency.is_some() {
        sell_currency = currency.unwrap();
    } else {
        sell_currency = XRD;
    }

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "create_proof_of_non_fungibles",
            manifest_args!(trader_key_resource, indexset![trader_key_local.clone()]),
        )
        .call_method(
            user.account,
            "withdraw_non_fungibles",
            manifest_args!(nft_address, indexset![nft_local_id.clone()]),
        )
        .take_all_from_worktop(nft_address, "listing")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                trader_component,
                "list",
                manifest_args!(lookup.bucket("listing"), sell_currency, price, auth_buyers,),
            )
        })
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

pub fn purchase_listing(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    marketplace_component: ComponentAddress,
    trader_component: ComponentAddress,
    nfgid: NonFungibleGlobalId,
    payment: Decimal,
    currency: Option<ResourceAddress>,
) {
    let buy_currency: ResourceAddress;

    if currency.is_some() {
        buy_currency = currency.unwrap();
    } else {
        buy_currency = XRD;
    }

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            user.account,
            "withdraw",
            manifest_args!(buy_currency, payment),
        )
        .take_all_from_worktop(buy_currency, "payment")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                marketplace_component,
                "purchase_listing",
                manifest_args!(
                    nfgid,
                    lookup.bucket("payment"),
                    trader_component,
                    user.account,
                ),
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
