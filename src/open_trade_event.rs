use crate::open_trader_account::Listing;
use scrypto::prelude::*;
/// This component acts as the central hub for all trade emitted events, such as listing creation, listing updates, listing cancellations, and listing purchases.

#[derive(ScryptoSbor, ScryptoEvent)]
struct ListingCreated {
    listing: Listing,
    nft_id: NonFungibleGlobalId,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct ListingUpdated {
    listing: Listing,
    nft_id: NonFungibleGlobalId,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct ListingCanceled {
    listing: Listing,
    nft_id: NonFungibleGlobalId,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct ListingPurchased {
    listing: Listing,
    nft_id: NonFungibleGlobalId,
}

#[blueprint]
#[events(ListingCreated, ListingUpdated, ListingCanceled, ListingPurchased)]
mod event {

    struct Event {
        emitter_badge_auth: ResourceAddress,
    }

    impl Event {
        pub fn create_event_listener(emitter_badge_auth: ResourceAddress) -> Global<Event> {
            let (event_address_reservation, _event_component_address) =
                Runtime::allocate_component_address(Event::blueprint_id());

            Self { emitter_badge_auth }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(event_address_reservation)
                .globalize()
        }

        pub fn listing_event(
            &self,
            listing: Listing,
            nft_id: NonFungibleGlobalId,
            emitter_badge: Proof,
        ) {
            emitter_badge.check(self.emitter_badge_auth);
            Runtime::emit_event(ListingCreated { listing, nft_id });
        }

        pub fn update_listing_event(
            &self,
            listing: Listing,
            nft_id: NonFungibleGlobalId,
            emitter_badge: Proof,
        ) {
            emitter_badge.check(self.emitter_badge_auth);
            Runtime::emit_event(ListingUpdated { listing, nft_id });
        }

        pub fn cancel_listing_event(
            &self,
            listing: Listing,
            nft_id: NonFungibleGlobalId,
            emitter_badge: Proof,
        ) {
            emitter_badge.check(self.emitter_badge_auth);
            Runtime::emit_event(ListingCanceled { listing, nft_id });
        }

        pub fn purchase_listing_event(
            &self,
            listing: Listing,
            nft_id: NonFungibleGlobalId,
            emitter_badge: Proof,
        ) {
            emitter_badge.check(self.emitter_badge_auth);
            Runtime::emit_event(ListingPurchased { listing, nft_id });
        }
    }
}
