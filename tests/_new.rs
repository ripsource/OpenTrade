// use scrypto::component::AnyComponent;
// use scrypto::data::manifest;
// use scrypto_test::prelude::*;

// use scrypto_test::utils::dump_manifest_to_file_system;
// use trader::open_trade_event::*;
// use trader::open_trade_factory::*;
// use trader::open_trader_account::*;
// use trader::royal_mint_example::*;

// #[test]
// fn integrated_test_purchase_royalty_nft() {
//     // Setup the environment
//     let mut ledger = LedgerSimulatorBuilder::new()
//         .without_kernel_trace()
//         .with_custom_genesis(CustomGenesis::default(
//             Epoch::of(1),
//             CustomGenesis::default_consensus_manager_config(),
//         ))
//         .build();

//     // Create an account
//     let (public_key, _private_key, account) = ledger.new_allocated_account();

//     // Publish package
//     let package_address = ledger.compile_and_publish(this_package!());

//     // Test the `instantiate_hello` function.
//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_function(
//             package_address,
//             "OpenHub",
//             "start_open_hub",
//             manifest_args!(),
//         )
//         .build();
//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );

//     println!("created open trade protocol");
//     receipt.expect_commit_success();
//     let component = receipt.expect_commit(true).new_component_addresses()[0];

//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_method(component, "fetch_virt_badge", manifest_args!())
//         .build();

//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("fetched virt badge address");
//     receipt.expect_commit_success();

//     let virt_badge: ResourceAddress = receipt.expect_commit(true).output(1);

//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_method(
//             component,
//             "fetch_royal_nft_depositer_badge",
//             manifest_args!(),
//         )
//         .build();

//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("fetched virt badge address");
//     receipt.expect_commit_success();
//     let depositer_badge: ResourceAddress = receipt.expect_commit(true).output(1);

//     let virt_badge: ResourceAddress = receipt.expect_commit(true).output(1);

//     // Test the `free_token` method.
//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_method(component, "create_open_trader", manifest_args!(account))
//         .call_method(
//             account,
//             "deposit_batch",
//             manifest_args!(ManifestExpression::EntireWorktop),
//         )
//         .build();
//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("created open trader account");

//     let (trader_key, _): (NonFungibleGlobalId, Bucket) = receipt.expect_commit(true).output(1);

//     let (trader_key_resource, trader_key_local): (ResourceAddress, NonFungibleLocalId) =
//         trader_key.clone().into_parts();

//     println!(
//         "nfgid: {:?}, resource: {:?}, local: {:?}",
//         trader_key, trader_key_resource, trader_key_local
//     );

//     receipt.expect_commit_success();

//     let trader_component = receipt.expect_commit(true).new_component_addresses()[0];

//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_function(
//             package_address,
//             "Event",
//             "create_event_listener",
//             manifest_args!(virt_badge),
//         )
//         .build();
//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("created event listener component");

//     receipt.expect_commit_success();

//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_function(
//             package_address,
//             "GenericMarketplace",
//             "start_marketplace",
//             manifest_args!(dec!(0.02)),
//         )
//         .call_method(
//             account,
//             "deposit_batch",
//             manifest_args!(ManifestExpression::EntireWorktop),
//         )
//         .build();
//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );

//     receipt.expect_commit_success();
//     println!("created generic marketplace component");

//     let marketplace_component = receipt.expect_commit(true).new_component_addresses()[0];

//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_method(
//             marketplace_component,
//             "get_marketplace_key_address",
//             manifest_args!(),
//         )
//         .build();
//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("got marketplace key address");

//     let marketplace_key: ResourceAddress = receipt.expect_commit(true).output(1);

//     let dapp_permissions: Vec<ComponentAddress> = vec![];
//     let buyer_permissions: Vec<ComponentAddress> = vec![];
//     let currencies: Vec<ResourceAddress> = vec![];
//     let minimum_amounts: HashMap<ResourceAddress, Decimal> = hashmap!();

//     let manifest = ManifestBuilder::new()
//         .lock_fee_from_faucet()
//         .call_function(
//             package_address,
//             "RoyalRascals",
//             "start_minting_rascals",
//             manifest_args!(
//                 dec!(100),
//                 XRD,
//                 1000u64,
//                 depositer_badge,
//                 dec!(0.05),
//                 dec!(0.1),
//                 false,
//                 false,
//                 false,
//                 false,
//                 dapp_permissions,
//                 buyer_permissions,
//                 currencies,
//                 minimum_amounts,
//                 false,
//             ),
//         )
//         .call_method(
//             account,
//             "deposit_batch",
//             manifest_args!(ManifestExpression::EntireWorktop),
//         )
//         .build();
//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("created royal rascals component");
//     receipt.expect_commit_success();

//     let rascal_component = receipt.expect_commit(true).new_component_addresses()[0];

//     println!("got here");

//     let map_of_data = vec![
//         hashmap! {
//             "trait_type".to_string() => "Hair".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Brown".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Eyes".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Blue".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Mouth".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Smile".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Nose".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Big".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Ears".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Small".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Hat".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "None".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Shirt".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Red".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Pants".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Blue".to_string(),
//         },
//         hashmap! {
//             "trait_type".to_string() => "Shoes".to_string(),
//             "trait_display".to_string() => "text".to_string(),
//             "trait_value".to_string() => "Black".to_string(),
//         },
//     ];
//     let mut data: Vec<(String, Vec<HashMap<String, String>>)> = vec![];

//     for i in 0..220 {
//         let number = i.to_string();

//         data.push((number, map_of_data.clone()));
//         println!("data: {:?}", i);
//     }

//     let manifest = ManifestBuilder::new()
//         .call_method(account, "lock_fee", manifest_args!(dec!(100)))
//         .call_method(rascal_component, "add_minting_data", manifest_args!(data))
//         .call_method(
//             account,
//             "deposit_batch",
//             manifest_args!(ManifestExpression::EntireWorktop),
//         )
//         .build();
//     let receipt = ledger.execute_manifest(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     receipt.expect_commit_success();
//     println!("receipt {:?}", receipt.fee_summary);
// }
