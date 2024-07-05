use scrypto::component::AnyComponent;
use scrypto_test::prelude::*;

use scrypto_test::utils::dump_manifest_to_file_system;
use trader::open_trade_event::*;
use trader::open_trade_factory::*;
use trader::open_trader_account::*;
use trader::royal_mint_example::*;

#[test]
fn integrated_test_purchase_royalty_nft() {
    // Setup the environment
    let mut ledger = LedgerSimulatorBuilder::new()
        .without_kernel_trace()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .build();

    // Create an account
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.compile_and_publish(this_package!());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "OpenHub",
            "start_open_hub",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("created open trade protocol");
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "fetch_virt_badge", manifest_args!())
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("fetched virt badge address");
    receipt.expect_commit_success();

    let virt_badge: ResourceAddress = receipt.expect_commit(true).output(1);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "fetch_royal_nft_depositer_badge",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("fetched virt badge address");
    receipt.expect_commit_success();
    let depositer_badge: ResourceAddress = receipt.expect_commit(true).output(1);

    let virt_badge: ResourceAddress = receipt.expect_commit(true).output(1);

    // Test the `free_token` method.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "create_open_trader", manifest_args!(account))
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("created open trader account");

    let (trader_key, _): (NonFungibleGlobalId, Bucket) = receipt.expect_commit(true).output(1);

    let (trader_key_resource, trader_key_local): (ResourceAddress, NonFungibleLocalId) =
        trader_key.clone().into_parts();

    println!(
        "nfgid: {:?}, resource: {:?}, local: {:?}",
        trader_key, trader_key_resource, trader_key_local
    );

    receipt.expect_commit_success();

    let trader_component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Event",
            "create_event_listener",
            manifest_args!(virt_badge),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("created event listener component");

    receipt.expect_commit_success();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "GenericMarketplace",
            "start_marketplace",
            manifest_args!(dec!(0.02)),
        )
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    println!("created generic marketplace component");

    let marketplace_component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            marketplace_component,
            "get_marketplace_key_address",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("got marketplace key address");

    let marketplace_key: ResourceAddress = receipt.expect_commit(true).output(1);

    let dapp_permissions: Vec<ComponentAddress> = vec![];
    let buyer_permissions: Vec<ComponentAddress> = vec![];
    let currencies: Vec<ResourceAddress> = vec![];
    let minimum_amounts: HashMap<ResourceAddress, Decimal> = hashmap!();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoyalNFTs",
            "start_minting_nft",
            manifest_args!(
                "Baked Potato NFTs".to_string(),
                "An Baked Potato NFT collection you can trade with royalties".to_string(),
                "https://www.allrecipes.com/thmb/c_2gXiAwkO6u1UJCY-1eAVCy0h0=/1500x0/filters:no_upscale():max_bytes(150000):strip_icc()/54679_perfect-baked-potato-Rita-1x1-1-91711252bb3740088c8ea55c5f9bef1c.jpg".to_string(),
                "https://www.onceuponachef.com/images/2022/11/baked-potatoes.jpg".to_string(),
                dec!(100),
                XRD,
                1000u64,
                depositer_badge,
                dec!(0.05),
                dec!(0.1),
                false,
                false,
                false,
                false,
                dapp_permissions,
                buyer_permissions,
                currencies,
                minimum_amounts,
                false,
            ),
        )
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("created royal rascals component");
    receipt.expect_commit_success();

    let rascal_component = receipt.expect_commit(true).new_component_addresses()[0];

    println!("got here");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(account, "withdraw", manifest_args!(XRD, dec!(100)))
        .take_all_from_worktop(XRD, "payment")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                rascal_component,
                "mint_preview_nft",
                manifest_args!(lookup.bucket("payment"), account),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    println!("minted rascal");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(rascal_component, "resource_address", manifest_args!())
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let rascal_address: ResourceAddress = receipt.expect_commit(true).output(1);
    let rascal_local_id: NonFungibleLocalId = NonFungibleLocalId::integer(0u64.into());
    let nfgid = NonFungibleGlobalId::new(rascal_address.clone(), rascal_local_id.clone());

    receipt.expect_commit_success();
    println!("got rascal address");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(trader_component, "fetch_auth_key", manifest_args!())
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let (trader_auth_resource, trader_auth_local): (ResourceAddress, NonFungibleLocalId) =
        receipt.expect_commit(true).output(1);

    receipt.expect_commit_success();
    println!(
        "{:?} got trader auth key, {:?} auth local",
        trader_auth_resource, trader_auth_local
    );

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            account,
            "create_proof_of_non_fungibles",
            manifest_args!(trader_key_resource, indexset![trader_key_local.clone()]),
        )
        // .pop_from_auth_zone("proof1")
        .call_method(
            account,
            "withdraw_non_fungibles",
            manifest_args!(rascal_address, indexset![rascal_local_id.clone()]),
        )
        .take_all_from_worktop(rascal_address, "rascal")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                trader_component,
                "royal_list",
                manifest_args!(
                    lookup.bucket("rascal"),
                    dec!(100),
                    XRD,
                    vec![marketplace_key.clone()],
                    // lookup.proof("proof1"),
                ),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();

    println!("{:?}", receipt.expect_commit(true));
    // advance time by atleast a second before a purchase can be made
    ledger.advance_to_round_at_timestamp(Round::of(2), 1718832354484);
    println!(
        "the time is: {:?}",
        ledger.get_current_time(TimePrecisionV2::Second)
    );
    ledger.advance_to_round_at_timestamp(Round::of(3), 1718832355484);
    println!(
        "the time is: {:?}",
        ledger.get_current_time(TimePrecisionV2::Second)
    );

    print!("listed rascal");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(account, "withdraw", manifest_args!(XRD, dec!(100)))
        .take_all_from_worktop(XRD, "rascal_payment")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                marketplace_component,
                "purchase_royal_listing",
                manifest_args!(
                    nfgid,
                    lookup.bucket("rascal_payment"),
                    trader_component,
                    account
                ),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();

    println!("purchased rascal");
    println!("{:?}", receipt);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            account,
            "create_proof_of_non_fungibles",
            manifest_args!(trader_key_resource, indexset![trader_key_local.clone()]),
        )
        // .pop_from_auth_zone("proof1")
        .call_method(
            account,
            "withdraw_non_fungibles",
            manifest_args!(rascal_address, indexset![rascal_local_id.clone()]),
        )
        .take_all_from_worktop(rascal_address, "rascal")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                trader_component,
                "royal_list",
                manifest_args!(
                    lookup.bucket("rascal"),
                    dec!(100),
                    XRD,
                    vec![marketplace_key.clone()],
                    // lookup.proof("proof1"),
                ),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    print!("listed rascal");
}

// #[test]
// fn test_hello_with_test_environment() -> Result<(), RuntimeError> {
//     // Arrange
//     let mut env = TestEnvironment::new();
//     let package_address =
//         PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

//     let mut hello = openhub::instantiate_hello(package_address, &mut env)?;

//     // Act
//     let bucket = hello.free_token(&mut env)?;

//     // Assert
//     let amount = bucket.amount(&mut env)?;
//     assert_eq!(amount, dec!("1"));

//     Ok(())
// }
