# In Live Development - Proof of Concept - not production ready, nor complete in any fashion.

This set of blueprints sets out the infrastructure for decentralised NFT trading on Radix, where 1 NFT can be listed on multiple marketplaces at the same time. The intention is to allow for users to have control over their assets, while ensuring marketplace aggregators can earn fees for their services and creators can enforce royalties in a flexible way. 

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
    - allow only users to hold your NFTs (High royalty enforcement)
    - allow users and dApps to hold your NFTs (Medium royalty enforcement)
    - remove all royalty restrictions (no royalty enforcement)
    - Switch between royalty restriction levels at any time
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
