#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Active,
    Released,
    Refunded,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Escrow {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
    pub release_time: u64,
}

#[contracttype]
pub enum DataKey {
    Escrow(u64),
    EscrowCount,
    Admin,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::EscrowCount, &0u64);
    }

    /// Create a new escrow
    pub fn create_escrow(
        env: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        release_time: u64,
    ) -> u64 {
        depositor.require_auth();
        assert!(amount > 0, "Amount must be greater than zero");

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::EscrowCount)
            .unwrap_or(0);
        let escrow_id = count + 1;

        let escrow = Escrow {
            depositor,
            beneficiary,
            token,
            amount,
            status: EscrowStatus::Active,
            release_time,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Escrow(escrow_id), &escrow);
        env.storage()
            .instance()
            .set(&DataKey::EscrowCount, &escrow_id);

        env.events()
            .publish((Symbol::new(&env, "escrow_created"),), (escrow_id,));

        escrow_id
    }

    /// Release funds to beneficiary
    pub fn release(env: Env, escrow_id: u64) {
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("Escrow not found");

        escrow.depositor.require_auth();
        assert!(
            escrow.status == EscrowStatus::Active,
            "Escrow is not active"
        );

        let current_time = env.ledger().timestamp();
        assert!(
            current_time >= escrow.release_time,
            "Release time has not been reached"
        );

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.beneficiary,
            &escrow.amount,
        );

        escrow.status = EscrowStatus::Released;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        env.events()
            .publish((Symbol::new(&env, "escrow_released"),), (escrow_id,));
    }

    /// Refund to depositor — admin only, works on Active or Disputed escrows
    pub fn refund(env: Env, escrow_id: u64) {
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("Escrow not found");

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        admin.require_auth();

        assert!(
            escrow.status == EscrowStatus::Active || escrow.status == EscrowStatus::Disputed,
            "Escrow cannot be refunded in current status"
        );

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.depositor,
            &escrow.amount,
        );

        escrow.status = EscrowStatus::Refunded;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        env.events()
            .publish((Symbol::new(&env, "escrow_refunded"),), (escrow_id,));
    }

    /// Get escrow details
    pub fn get_escrow(env: Env, escrow_id: u64) -> Escrow {
        env.storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("Escrow not found")
    }

    /// Get total escrow count
    pub fn get_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::EscrowCount)
            .unwrap_or(0)
    }

    /// Raise a dispute — beneficiary only
    pub fn dispute(env: Env, escrow_id: u64) {
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("Escrow not found");

        escrow.beneficiary.require_auth();
        assert!(
            escrow.status == EscrowStatus::Active,
            "Escrow is not active"
        );

        escrow.status = EscrowStatus::Disputed;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        env.events()
            .publish((Symbol::new(&env, "escrow_disputed"),), (escrow_id,));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env,
    };

    fn create_token(env: &Env, admin: &Address) -> Address {
        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        token_contract.address()
    }

    #[test]
    fn test_create_escrow() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = create_token(&env, &admin);
        StellarAssetClient::new(&env, &token).mint(&depositor, &1000);

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        let escrow_id = client.create_escrow(&depositor, &beneficiary, &token, &100, &0u64);
        assert_eq!(escrow_id, 1);
        assert_eq!(client.get_escrow(&escrow_id).status, EscrowStatus::Active);
        assert_eq!(client.get_count(), 1);
    }

    #[test]
    fn test_release() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = create_token(&env, &admin);
        StellarAssetClient::new(&env, &token).mint(&depositor, &1000);

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        let escrow_id = client.create_escrow(&depositor, &beneficiary, &token, &100, &0u64);
        client.release(&escrow_id);
        assert_eq!(client.get_escrow(&escrow_id).status, EscrowStatus::Released);
    }

    #[test]
    fn test_dispute_then_refund() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = create_token(&env, &admin);
        StellarAssetClient::new(&env, &token).mint(&depositor, &1000);

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        let escrow_id = client.create_escrow(&depositor, &beneficiary, &token, &100, &0u64);
        client.dispute(&escrow_id);
        assert_eq!(client.get_escrow(&escrow_id).status, EscrowStatus::Disputed);

        client.refund(&escrow_id);
        assert_eq!(client.get_escrow(&escrow_id).status, EscrowStatus::Refunded);
    }

    #[test]
    #[should_panic(expected = "Amount must be greater than zero")]
    fn test_zero_amount_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = create_token(&env, &admin);

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        client.create_escrow(&depositor, &beneficiary, &token, &0, &0u64);
    }

    #[test]
    fn test_release_time_enforced() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(500);

        let admin = Address::generate(&env);
        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = create_token(&env, &admin);
        StellarAssetClient::new(&env, &token).mint(&depositor, &1000);

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        // release_time is 1000, current time is 500 — should fail
        let escrow_id = client.create_escrow(&depositor, &beneficiary, &token, &100, &1000u64);
        let result = client.try_release(&escrow_id);
        assert!(result.is_err());
    }
}
