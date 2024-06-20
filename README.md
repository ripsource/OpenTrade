# In Live Development - Proof of Concept - not production ready, nor complete in any fashion.

This set of blueprints sets out the infrastructure for decentralised NFT trading on Radix, where 1 NFT can be listed on multiple marketplaces at the same time. The intention is to allow for users to have control over their assets, while ensuring marketplace aggregators can earn fees for their services and creators can enforce royalties in a flexible way. 

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
    - Discount all your prices at once for a flash sale
    - Track your sales history across marketplaces and private deals
    - Use your NFTs even when they are listed (proof creation)
- Private trades between users supported
- Trove style swaps of multiple assets supported
- 0 fees for private trades and Trove style swaps

## For NFT Creators
- Any existing collection or new collection no matter how it is minted is compatible with this system
- %fee Royalties are supported for newly minted collections
- Use a standard template to mint collections that are fully integrated with the trading system
- BYOB - Bring your own blueprint, integration to your collection requires only a few lines of Scrypto
- Configure royalty enforcement level at any time:
    - allow only users to hold your NFTs + select specific dApps that can hold/interact with your NFTs (High royalty enforcement)
    - allow users and any dApps to hold your NFTs (Medium royalty enforcement)
    - remove all royalty restrictions (no royalty enforcement)
    - Switch between royalty restriction levels at any time
    - Allows users to transfer royalty-enforced NFTs between accounts they can demonstrate they own for free
    - Charge % fees or flat fees
    - Set royalty configuration parameter such as maximum %fee
    - customise fee % at any time 
    - Lock royalty configuration to give traders confidence

## For marketplaces
- 0 fees for marketplaces to integrate 
- Easy setup of your own marketplace, requiring just one blueprint
- Robust way to charge additional service fees
- Easy aggregation of trade activity
- No managing of multiple badges/components
- No calculation/compute required for royalty-enforced NFTs
