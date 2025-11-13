#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to store community currency details
#[contracttype]
#[derive(Clone)]
pub struct CurrencyInfo {
    pub name: String,
    pub symbol: String,
    pub total_supply: i128,
    pub community_admin: Address,
}

// Structure to track user balances
#[contracttype]
pub enum Balance {
    User(Address)
}

// Key for storing currency information
const CURRENCY_INFO: Symbol = symbol_short!("CURR_INFO");

// Key for tracking if currency is initialized
const INITIALIZED: Symbol = symbol_short!("INIT");

#[contract]
pub struct CommunityCurrencyContract;

#[contractimpl]
impl CommunityCurrencyContract {

    // Initialize the community currency with name, symbol, and initial supply
    pub fn initialize(
        env: Env, 
        admin: Address,
        name: String, 
        symbol: String, 
        initial_supply: i128
    ) -> bool {
        
        // Check if already initialized
        let is_initialized: bool = env.storage().instance().get(&INITIALIZED).unwrap_or(false);
        
        if is_initialized {
            log!(&env, "Currency already initialized!");
            panic!("Currency already initialized!");
        }

        // Require admin authentication
        admin.require_auth();

        // Create currency info
        let currency = CurrencyInfo {
            name: name.clone(),
            symbol: symbol.clone(),
            total_supply: initial_supply,
            community_admin: admin.clone(),
        };

        // Store currency info
        env.storage().instance().set(&CURRENCY_INFO, &currency);
        
        // Set admin's initial balance to total supply
        env.storage().instance().set(&Balance::User(admin.clone()), &initial_supply);
        
        // Mark as initialized
        env.storage().instance().set(&INITIALIZED, &true);
        
        env.storage().instance().extend_ttl(100000, 100000);

        log!(&env, "Community Currency '{}' initialized with supply: {}", symbol, initial_supply);
        
        true
    }

    // Transfer tokens from one user to another
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> bool {
        
        // Require authentication from sender
        from.require_auth();

        // Validate amount
        if amount <= 0 {
            log!(&env, "Transfer amount must be positive!");
            panic!("Invalid amount!");
        }

        // Get sender balance
        let from_balance: i128 = env.storage().instance()
            .get(&Balance::User(from.clone()))
            .unwrap_or(0);

        // Check sufficient balance
        if from_balance < amount {
            log!(&env, "Insufficient balance!");
            panic!("Insufficient balance!");
        }

        // Get receiver balance
        let to_balance: i128 = env.storage().instance()
            .get(&Balance::User(to.clone()))
            .unwrap_or(0);

        // Update balances
        env.storage().instance().set(&Balance::User(from.clone()), &(from_balance - amount));
        env.storage().instance().set(&Balance::User(to.clone()), &(to_balance + amount));

        env.storage().instance().extend_ttl(100000, 100000);

        log!(&env, "Transferred {} tokens from {:?} to {:?}", amount, from, to);

        true
    }

    // Get balance of a specific user
    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage().instance()
            .get(&Balance::User(user.clone()))
            .unwrap_or(0)
    }

    // Get currency information
    pub fn get_currency_info(env: Env) -> CurrencyInfo {
        env.storage().instance()
            .get(&CURRENCY_INFO)
            .unwrap_or(CurrencyInfo {
                name: String::from_str(&env, "Not_Initialized"),
                symbol: String::from_str(&env, "N/A"),
                total_supply: 0,
                community_admin: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CommunityCurrencyContract);
        let client = CommunityCurrencyContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let name = String::from_str(&env, "Community Token");
        let symbol = String::from_str(&env, "COMM");
        let supply = 1000000;

        let result = client.initialize(&admin, &name, &symbol, &supply);
        assert_eq!(result, true);
    }

    #[test]
    fn test_transfer() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CommunityCurrencyContract);
        let client = CommunityCurrencyContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        
        // Initialize currency
        client.initialize(&admin, 
            &String::from_str(&env, "Community Token"), 
            &String::from_str(&env, "COMM"), 
            &1000000);

        // Transfer tokens
        client.transfer(&admin, &user1, &500);

        // Check balances
        assert_eq!(client.get_balance(&admin), 999500);
        assert_eq!(client.get_balance(&user1), 500);
    }

    #[test]
    fn test_get_balance() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CommunityCurrencyContract);
        let client = CommunityCurrencyContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let balance = client.get_balance(&user);
        
        assert_eq!(balance, 0); // Should return 0 for uninitialized balance
    }
}