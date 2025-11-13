#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, String, Symbol, symbol_short};

// Struct to store artwork details
#[contracttype]
#[derive(Clone)]
pub struct Artwork {
    pub art_id: u64,
    pub owner: Address,
    pub title: String,
    pub price_per_day: u64,
    pub is_available: bool,
}

// Struct to store rental details
#[contracttype]
#[derive(Clone)]
pub struct Rental {
    pub rental_id: u64,
    pub art_id: u64,
    pub renter: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
}

// Storage keys
#[contracttype]
pub enum ArtworkBook {
    Artwork(u64)
}

#[contracttype]
pub enum RentalBook {
    Rental(u64)
}

const ART_COUNT: Symbol = symbol_short!("ART_CNT");
const RENTAL_COUNT: Symbol = symbol_short!("RENT_CNT");

#[contract]
pub struct ArtRentalContract;

#[contractimpl]
impl ArtRentalContract {

    // Function to register a new artwork for rental
    pub fn register_artwork(
        env: Env, 
        owner: Address, 
        title: String, 
        price_per_day: u64
    ) -> u64 {
        owner.require_auth();
        
        let mut art_count: u64 = env.storage().instance().get(&ART_COUNT).unwrap_or(0);
        art_count += 1;
        
        let artwork = Artwork {
            art_id: art_count,
            owner: owner.clone(),
            title,
            price_per_day,
            is_available: true,
        };
        
        env.storage().instance().set(&ArtworkBook::Artwork(art_count), &artwork);
        env.storage().instance().set(&ART_COUNT, &art_count);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Artwork registered with ID: {}", art_count);
        art_count
    }

    // Function to rent an artwork
    pub fn rent_artwork(
        env: Env,
        art_id: u64,
        renter: Address,
        duration_days: u64
    ) -> u64 {
        renter.require_auth();
        
        let mut artwork = Self::view_artwork(env.clone(), art_id);
        
        if !artwork.is_available {
            log!(&env, "Artwork is not available for rent");
            panic!("Artwork is not available for rent");
        }
        
        let mut rental_count: u64 = env.storage().instance().get(&RENTAL_COUNT).unwrap_or(0);
        rental_count += 1;
        
        let start_time = env.ledger().timestamp();
        let end_time = start_time + (duration_days * 86400); // 86400 seconds in a day
        
        let rental = Rental {
            rental_id: rental_count,
            art_id,
            renter: renter.clone(),
            start_time,
            end_time,
            is_active: true,
        };
        
        // Mark artwork as unavailable
        artwork.is_available = false;
        env.storage().instance().set(&ArtworkBook::Artwork(art_id), &artwork);
        
        env.storage().instance().set(&RentalBook::Rental(rental_count), &rental);
        env.storage().instance().set(&RENTAL_COUNT, &rental_count);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Artwork rented with Rental ID: {}", rental_count);
        rental_count
    }

    // Function to return an artwork after rental period
    pub fn return_artwork(env: Env, rental_id: u64) {
        let mut rental = Self::view_rental(env.clone(), rental_id);
        
        if !rental.is_active {
            log!(&env, "Rental is not active");
            panic!("Rental is not active");
        }
        
        rental.is_active = false;
        env.storage().instance().set(&RentalBook::Rental(rental_id), &rental);
        
        // Mark artwork as available again
        let mut artwork = Self::view_artwork(env.clone(), rental.art_id);
        artwork.is_available = true;
        env.storage().instance().set(&ArtworkBook::Artwork(rental.art_id), &artwork);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Artwork returned successfully");
    }

    // View function to get artwork details
    pub fn view_artwork(env: Env, art_id: u64) -> Artwork {
        env.storage().instance().get(&ArtworkBook::Artwork(art_id)).unwrap_or(Artwork {
            art_id: 0,
            owner: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            title: String::from_str(&env, "Not_Found"),
            price_per_day: 0,
            is_available: false,
        })
    }

    // View function to get rental details
    pub fn view_rental(env: Env, rental_id: u64) -> Rental {
        env.storage().instance().get(&RentalBook::Rental(rental_id)).unwrap_or(Rental {
            rental_id: 0,
            art_id: 0,
            renter: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            start_time: 0,
            end_time: 0,
            is_active: false,
        })
    }
}
