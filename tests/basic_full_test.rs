use scrypto_test::prelude::*;
mod common;
mod creator_manifests;
mod marketplace_manifests;
mod misc_manifests;
mod scenario_manifests;
mod trader_manifests;
use common::*;
use creator_manifests::*;
use marketplace_manifests::*;
use misc_manifests::*;
use scenario_manifests::*;
use trader_manifests::*;

#[test]
fn list_and_purchase_royalty_nft() {
    let (mut test_runner, user, package) = setup_for_test();

    let open_hub_component = instantiate_open_hub(&mut test_runner, &user, package);

    let virtual_badge = fetch_virt_badge(&mut test_runner, &user, open_hub_component.clone());

    let depositer_badger = fetch_depositer_badge(&mut test_runner, &user, open_hub_component);

    let (_trader_key_resource, _trader_key_local, trader_component) =
        create_open_trader(&mut test_runner, &user, open_hub_component);

    create_event_listener(&mut test_runner, &user, package, virtual_badge.clone());

    // let (marketplace_component, marketplace_key) =
    //     create_marketplace(&mut test_runner, &user, package, dec!(0.02));

    // let royalty_config = defaults_royalty_config();

    // let (royalty_nft_component, creator_key) = create_royalty_nft(
    //     &mut test_runner,
    //     &user,
    //     package,
    //     royalty_config,
    //     depositer_badger.clone(),
    // );

    // enable_mint_reveal(&mut test_runner, &user, royalty_nft_component, creator_key);

    // mint_royalty_nft(&mut test_runner, &user, royalty_nft_component);

    // let nft_address = nft_address(&mut test_runner, &user, royalty_nft_component);

    // let global_id = create_global_id(nft_address.clone(), 0);

    // let (trader_auth_resource, trader_auth_local) =
    //     trader_auth_key(&mut test_runner, &user, trader_component.clone());

    // list_royalty_nft(
    //     &mut test_runner,
    //     &user,
    //     trader_component.clone(),
    //     trader_auth_resource.clone(),
    //     trader_auth_local.clone(),
    //     nft_address.clone(),
    //     NonFungibleLocalId::integer(0),
    //     dec!(100),
    //     None,
    //     vec![marketplace_key.clone()],
    // );

    // purchase_royalty_nft(
    //     &mut test_runner,
    //     &user,
    //     marketplace_component,
    //     trader_component,
    //     global_id,
    //     dec!(100),
    //     None,
    // )
}
