use scrypto::prelude::*;
use scrypto_test::prelude::*;

use crate::common::*;

#[derive(ScryptoSbor)]
pub struct RoyaltyConfig {
    /// The royalty percentage to be paid to the creator of the Royal NFTs (e.g. 0.1 = 10% - maximum value is 1)
    pub royalty_percent: Decimal,
    /// The maximum royalty percentage that can be set - once set can not be increased. It can be decreased though.
    pub maximum_royalty_percent: Decimal,
    /// Offers an option for a creator to only allow trading of their assets in certain currencies (currencies selected in the permitted_currencies field)
    pub limit_currencies: bool,
    /// Currencies that the creator can receive royalties in/an NFT can be traded in (e.g. XRD)
    pub permitted_currencies: Vec<ResourceAddress>,
    /// Set minimum fixed amounts of royalties for each permitted currency
    /// this is useful if a creator wants to allow private sales, but still ensure they receive royalties.
    pub minimum_royalties: bool,
    /// Minimum royalty amounts for each currency
    pub minimum_royalty_amounts: HashMap<ResourceAddress, Decimal>,
    // Permissioned dApps - Dapps that you want to allow your NFTs to interact with/be deposited to.
    pub limit_dapps: bool,
    /// A permission list of components an NFT can be transferred to
    pub permissioned_dapps: Vec<ComponentAddress>,
    /// This is useful because private traders could trade the NFTs without paying royalties, so this closes that loophole.
    /// However, this can be turned off if the creator wants to allow any trader to trade the NFTs. If a creator wants to allow private sales,
    /// but still receive royalties - they can set a minimum royalty amount for each currency.
    pub limit_buyers: bool,
    /// A permission list for marketplaces/individual buyers that can trade the NFTs
    /// This requires that a certain badge is shown by the buyer or marketplace in order to purchase an NFT.
    pub permissioned_buyers: Vec<ComponentAddress>,
    /// lock royalty configuration: Option can give traders confidence that the royalty percentage/settings will not change.
    /// There's no method to undo this once set to true. However, right now creators can always take steps to make their
    /// royalties more relaxed even if locked - i.e. remove mininimum royalties, allow all buyers, etc.
    pub royalty_configuration_locked: bool,
}

pub fn instantiate_open_hub(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    package: PackageAddress,
) -> ComponentAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "OpenHub", "start_open_hub", manifest_args!())
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

    receipt.expect_commit_success().new_component_addresses()[0]
}

pub fn fetch_virt_badge(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
) -> ResourceAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "fetch_virt_badge", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit(true).output(1)
}

pub fn fetch_depositer_badge(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
) -> ResourceAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "fetch_royal_nft_depositer_badge",
            manifest_args!(),
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

    receipt.expect_commit(true).output(1)
}

pub fn create_open_trader(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    component: ComponentAddress,
) -> (ResourceAddress, NonFungibleLocalId, ComponentAddress) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "create_open_trader",
            manifest_args!(user.account),
        )
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

    let (trader_key, _): (NonFungibleGlobalId, Bucket) = receipt.expect_commit(true).output(1);
    let (trader_key_resource, trader_key_local): (ResourceAddress, NonFungibleLocalId) =
        trader_key.clone().into_parts();

    let trader_component = receipt.expect_commit(true).new_component_addresses()[0];

    (trader_key_resource, trader_key_local, trader_component)
}

pub fn create_event_listener(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    package: PackageAddress,
    virt_badge_resource: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "Event",
            "create_event_listener",
            manifest_args!(virt_badge_resource),
        )
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    };
}

pub fn create_marketplace(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    package: PackageAddress,
    fee: Decimal,
) -> (ComponentAddress, ResourceAddress) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "GenericMarketplace",
            "start_marketplace",
            manifest_args!(fee),
        )
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

    let component = receipt.expect_commit_success().new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component.clone(),
            "get_marketplace_key_address",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );
    println!("got marketplace key address");

    let marketplace_key: ResourceAddress = receipt.expect_commit(true).output(1);

    (component, marketplace_key)
}

pub fn defaults_royalty_config() -> RoyaltyConfig {
    let dapp_permissions: Vec<ComponentAddress> = vec![];
    let buyer_permissions: Vec<ComponentAddress> = vec![];
    let currencies: Vec<ResourceAddress> = vec![];
    let minimum_amounts: HashMap<ResourceAddress, Decimal> = hashmap!();

    RoyaltyConfig {
        royalty_percent: Decimal::from_str("0.1").unwrap(),
        maximum_royalty_percent: Decimal::from_str("0.5").unwrap(),
        limit_currencies: false,
        permitted_currencies: currencies,
        minimum_royalties: false,
        minimum_royalty_amounts: minimum_amounts,
        limit_dapps: false,
        permissioned_dapps: dapp_permissions,
        limit_buyers: false,
        permissioned_buyers: buyer_permissions,
        royalty_configuration_locked: false,
    }
}

pub fn custom_royalty_config(
    royalty_percent: Decimal,
    max_royalty_percent: Decimal,
    limit_currencies: bool,
    permitted_currencies: Vec<ResourceAddress>,
    minimum_royalties: bool,
    minimum_royalty_amounts: HashMap<ResourceAddress, Decimal>,
    limit_dapps: bool,
    permissioned_dapps: Vec<ComponentAddress>,
    limit_buyers: bool,
    permissioned_buyers: Vec<ComponentAddress>,
    royalty_configuration_locked: bool,
) -> RoyaltyConfig {
    RoyaltyConfig {
        royalty_percent,
        maximum_royalty_percent: max_royalty_percent,
        limit_currencies,
        permitted_currencies,
        minimum_royalties,
        minimum_royalty_amounts,
        limit_dapps,
        permissioned_dapps,
        limit_buyers,
        permissioned_buyers,
        royalty_configuration_locked,
    }
}

pub fn create_mint_factory(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    package: PackageAddress,
) -> ComponentAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "MintFactory",
            "start_mint_factory",
            manifest_args!(),
        )
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

    let component = receipt.expect_commit_success().new_component_addresses()[0];

    component
}

pub fn create_royalty_nft_direct(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    package: PackageAddress,
    royalty_config: RoyaltyConfig,
    depositer_badge: ResourceAddress,
) -> (ComponentAddress, ResourceAddress) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
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
                vec![false, false, true, false, false],
                depositer_badge,
                true,
                royalty_config.royalty_percent,
                royalty_config.maximum_royalty_percent,
                royalty_config.limit_buyers,
                royalty_config.limit_currencies,
                royalty_config.limit_dapps,
                royalty_config.minimum_royalties,
                royalty_config.permissioned_dapps,
                royalty_config.permissioned_buyers,
                royalty_config.permitted_currencies,
                royalty_config.minimum_royalty_amounts,
            ),
        )
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

    let component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component.clone(), "creator_admin", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    let creator_key: ResourceAddress = receipt.expect_commit(true).output(1);

    (component, creator_key)
}

pub fn create_royalty_nft(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    mint_factory_component: ComponentAddress,
    royalty_config: RoyaltyConfig,
    depositer_badge: ResourceAddress,
) -> (ComponentAddress, ResourceAddress) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            mint_factory_component,
            "create_royal_nft",
            manifest_args!(
                "Baked Potato NFTs".to_string(),
                "An Baked Potato NFT collection you can trade with royalties".to_string(),
                "https://www.allrecipes.com/thmb/c_2gXiAwkO6u1UJCY-1eAVCy0h0=/1500x0/filters:no_upscale():max_bytes(150000):strip_icc()/54679_perfect-baked-potato-Rita-1x1-1-91711252bb3740088c8ea55c5f9bef1c.jpg".to_string(),
                "https://www.onceuponachef.com/images/2022/11/baked-potatoes.jpg".to_string(),
                dec!(100),
                XRD,
                1000u64,
                vec![false, false, true, false, false],
                depositer_badge,
                true,
                royalty_config.royalty_percent,
                royalty_config.maximum_royalty_percent,
                royalty_config.limit_buyers,
                royalty_config.limit_currencies,
                royalty_config.limit_dapps,
                royalty_config.minimum_royalties,
                royalty_config.permissioned_dapps,
                royalty_config.permissioned_buyers,
                royalty_config.permitted_currencies,
                royalty_config.minimum_royalty_amounts,
            ),
        )
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

    let component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component.clone(), "creator_admin", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    let creator_key: ResourceAddress = receipt.expect_commit(true).output(1);

    (component, creator_key)
}

pub fn create_standard_nft(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    package: PackageAddress,
    royalty_config: RoyaltyConfig,
    depositer_badge: ResourceAddress,
) -> (ComponentAddress, ResourceAddress) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
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
                vec![false, false, true, false, false],
                depositer_badge,
                false,
                royalty_config.royalty_percent,
                royalty_config.maximum_royalty_percent,
                royalty_config.limit_buyers,
                royalty_config.limit_currencies,
                royalty_config.limit_dapps,
                royalty_config.minimum_royalties,
                royalty_config.permissioned_dapps,
                royalty_config.permissioned_buyers,
                royalty_config.permitted_currencies,
                royalty_config.minimum_royalty_amounts,
            ),
        )
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

    let component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component.clone(), "creator_admin", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    let creator_key: ResourceAddress = receipt.expect_commit(true).output(1);

    (component, creator_key)
}

pub fn create_generic_dapp(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    package: PackageAddress,
) -> ComponentAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "Dapp", "start_dapp", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit_success().new_component_addresses()[0]
}

pub fn withdraw_royalty_nft(
    test_runner: &mut DefaultLedgerSimulator,
    user: &User,
    dapp_component: ComponentAddress,
    trader_account: ComponentAddress,
    nft_address: ResourceAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            dapp_component,
            "withdraw_royalty_nft",
            manifest_args!(nft_address, trader_account),
        )
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
