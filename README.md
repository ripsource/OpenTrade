![alt text](https://github.com/ripsource/OpenTrade/blob/main/img/opentrade.png?raw=true)


# Free and Open Source NFT Creator, Trader, Marketplace Protocol - _In Live Development_ 

### Try on stokenet: component_tdx_2_1cpark7wnjkk5qv0rgt6qejr9ny6hteck47asj7snz8plme6zjkafwp

Create a new trade account (example):
```
CALL_METHOD
    Address("component_tdx_2_1cpark7wnjkk5qv0rgt6qejr9ny6hteck47asj7snz8plme6zjkafwp")
    "create_open_trader"
    Address("account_tdx_2_12y0xpdypsltq90w07lwnlh2640tg28m8v0cg2yppzlhgnwpndhq47c")
;
CALL_METHOD
    Address("account_tdx_2_12y0xpdypsltq90w07lwnlh2640tg28m8v0cg2yppzlhgnwpndhq47c")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

Create a marketplace (example):
```
CALL_FUNCTION
    Address("package_tdx_2_1p5jfw2yyg0xzmjlq5ez8ayk83faz6rmj8n4wwh0mgqv4e8zujnx0dj")
    "GenericMarketplace"
    "start_marketplace"
    Decimal("0.02") // set your marketplace fee
;
CALL_METHOD
    Address("account_tdx_2_12y0xpdypsltq90w07lwnlh2640tg28m8v0cg2yppzlhgnwpndhq47c")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```


This set of blueprints sets out the infrastructure for decentralised NFT trading on Radix, where 1 NFT can be listed on multiple marketplaces at the same time. The intention is to allow for users to have control over their assets, while ensuring marketplace aggregators can earn fees for their services and creators can enforce royalties in a highly configurable way. 

You'll find basic blueprints for:
- A shared-escrow account for individual traders to use for listing their NFTs as well as for purchasers to buy from.
- A factory component that creates trading accounts and provides them with specific auth badges
- A basic minting script for a Royalty-Enforced NFT collection which for simplicity also serves as royalty collection and configuration component for a creator
- An example of a generic marketplace: As all trading logic is embedded in trading accounts, very little is required here
- An example of generic dApp: solely serving the purpose of demonstrating how a royalty-enforced NFT could be transferred to a permitted dApp
- An event emitter component: This centralised component links in to all trader accounts and emits events for listings/purchases/cancellations/updates.

The design hopes to boast the following features:

## For traders:
- Control your listings all from one place
    - List once, list everywhere
    - Change prices without re-listing
    - Make offers with the same tokens on multiple NFTs at the same time
    - Easily track your sales history across marketplaces and private deals
    - Use your NFTs even when they are listed (proof creation)
- Private trades between users supported
- Trove style swaps of multiple assets supported
- 0 fees for private trades and Trove style swaps

## For NFT Creators

- Any existing collection or new collection no matter how it is minted is compatible with this system
- It is completely free to set up and manage
- There is 0 lock in. Something better comes along? No problem, no fee.
- Equally, if OpenTrade gets better/adds new features - creators can upgrade to them easily too
- %fee Royalties are supported for newly minted collections
- Use a standard template to mint collections that are fully integrated with the trading system
- BYOB - Bring your own blueprint, integration to your collection requires only a few lines of Scrypto
- Add advanced reactive functionality to your NFTs:
    - Have NFTs metadata react to being listed or sold
    - Trigger things when NFTs are sold for certain prices
    - Make NFTs reactive to their interactions with dApps (e.g. metadata that counts how many times they've been loaned, used in a game, etc.)
- Create unqiue royalty configurations:
    - allow only accounts to hold your NFTs + select specific dApps that can hold/interact with your NFTs (High royalty enforcement)
    - allow users and any dApps to hold your NFTs (Medium royalty enforcement)
    - remove all royalty restrictions (no royalty enforcement)
    - Switch between royalty restriction levels at any time
    - Allows users to transfer royalty-enforced NFTs between accounts they can demonstrate they own for free
    - Charge % fees or flat fees
    - Set royalty configuration parameter such as maximum %fee
    - customise fee % at any time 
    - Lock royalty configuration to give traders confidence
    - Select only specific currencies to receive royalties in
    - Create minimum royalty thresholds for selected currencies

## For marketplaces
- 0 fees for marketplaces to integrate 
- Easy setup of your own marketplace, requiring just one blueprint
- Robust way to charge additional service fees
- Easy aggregation of trade activity
- No managing of multiple badges/components
- No calculation/compute required for royalty-enforced NFTs


# A novel approach to royalties
In order to impose royalties on NFT trades, we have to introduce restrictions on either/or both the deposit rules and withdraw rules of an NFT. This means requiring some form of authorisation before a NFT can be deposited into a vault or withdrawn from one. Typically to handle this, you might have a single component that generates proofs of badges that allow the action to take place, however proofs can be cloned to be used multiple times, they're not very specific for what they can allow and ultimately, lead to a royalty system with several easy ways to bypass payments. 

The Open Trade system instead gives authority via a badge to every single trading account to be able to deposit royalty-enforced NFTs. The badge acts as a universal admin for deposit rules across every Royalty NFT created on the system. While this may seem counter-intuitive at first as our goal is to not let individual traders have the power to bypass these restrictions, in fact because the badge is locked in component vault that can only be accessed in specific ways that we dictate, there's no way for a user to gain access to the badge - it is kept and used entirely virtually, never becoming exposed to control by the user.

When a user wants to list a Royalty NFT to their trading account, they simply withdraw it and their account authorising depositing it into their trading account. When a user/marketplace purchases the NFT, they use the purchase_royal_listing method which checks the metadata of the NFT for a royalty component and sends the full payment to the royalty component for processing before allowing the trading account to then deposit the NFT in the buyers account. This pattern allows for a high level of configurability of how NFT royalties are collected as well as opening the possibility for dynamic reactive methods based on trading/transferring of an NFT. 

Royalty systems often have to make comprimises between the level of enforcement and interoperability/usability (i.e. can people transfer the NFT between their own accounts for no charge? or is that a way to bypass royalties?). The Open Trader standard closed many common loopholes while also allowing creators to customise their royalties for the right balance of enforcement and freedom. The implementation offers a range of options that can be enabled to create a very strict level of royalty enforcement, or certain areas can be relaxed depending on what a creator wants.

## Common royalty loopholes and solutions

### Smuggling assets 
Smuggling is where someone places a Royalty NFT into some form of wrapper - this might be a component such as a Soulstore where whoever owns the badge to access the soulstore, owns all the assets inside of the component. Once the NFT is inside this wrapper, people can then just trade the badge to the soulstore without paying any royalties as the Royalty NFT hasn't actually moved. 

_Solution:_ Users can create a trusted permission list of dApps that their NFTs can be used with and modify this when needed to add new dApps or remove ones that are bypassing royalties. Equally, a creator can enforce that simply only Accounts on Radix can ever hold the NFT.

### Atomic fee skipping
Due to the atomic guarantees and composable manifests on Radix, its possible that a user could list a Royalty NFT for sale and then purchase it to send to another user all within the same transaction. This would allow a user to do a private deal where they list the NFT for 0 XRD, but don't risk anyone 'sniping' it because they also purchase it to another account at the same time. 

_Solution_: Similar to the FAUCET blueprint, we store a hash of the listing tx at runtime and check against this when a purchase method is called. Thank you to Yo for suggesting this. 

### Can you think of another way you could avoid royalties in this system?
Most common loopholes appear to be addressed by this system if all the settings are configured to their strictest level by a creator in the Royalty Conig, but if you can think of another - I'll give you US $50 - or $100 if you also have the solution... And swapping with someone for their seed phrases/private keys is funny, but not in scope here. 

## Future areas for development and improvement

- As Royalty-Enforced NFTs are deposit restricted, we have to make direct deposit calls from within Scrypto methods. Currently, the Radix wallet only support 1-layer of transaction information in the GUI manifests and therefore, royalty NFTs don't show up as being depoisted to the user's account. Once we have Allowances, we should be able to update this - however, in the meantime, a temporary solution is to mint a receipt that gives the appearance of the NFT being purchased so that the user can visually verify the NFT that's being deposited to them.
- Currently the basic minting and royalty component example blueprint is quite cumbersome because of its 'feature richness', there is likely some more standardised types of minting examples that could be created for project developers to easily modify what they want - rather than having to grapple with all the details at once.
- Currently there is no support for multi-listings of the royalty NFTs. While mult-listing of royalty NFTs from the same collection seems straight forward - A decision would need to be made as to whether NFTs from different collections could be listed together and enjoy their respective royalties for the full payment amount.
- While you can create a listing in any currency, it would be possible to add features that allow a listing in multiple currencies/respective amounts. i.e. You can have a base price in XRD, then a discounted price if paying with USDC.




