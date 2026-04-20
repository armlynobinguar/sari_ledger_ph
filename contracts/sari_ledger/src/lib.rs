#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------
// Instance storage holds hot state (owner, counters, total revenue).
// Persistent storage holds long-lived records (inventory, sales, loans).
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Owner,                  // Address of the store owner
    TotalRevenue,           // Running sum of all sale proceeds (i128)
    SaleCount,              // Auto-incrementing sale id
    LoanCount,              // Auto-incrementing loan id
    Inventory(Symbol),      // product_id -> InventoryItem
    Sale(u32),              // sale_id -> Sale
    Loan(u32),              // loan_id -> Loan
}

// An SKU the owner stocks. avg_cost tracks weighted-average unit cost so
// the owner (or a lender) can compute gross margin off-chain later.
#[contracttype]
#[derive(Clone)]
pub struct InventoryItem {
    pub quantity: u32,
    pub avg_cost: i128,
}

// A single sale event — the append-only spine of the revenue history.
#[contracttype]
#[derive(Clone)]
pub struct Sale {
    pub product_id: Symbol,
    pub quantity: u32,
    pub price: i128,
    pub timestamp: u64,
}

// A loan drawn against documented revenue. `repaid` increases over time
// as the owner pays back; a loan is considered closed when repaid >= amount.
#[contracttype]
#[derive(Clone)]
pub struct Loan {
    pub amount: i128,
    pub repaid: i128,
    pub timestamp: u64,
}

// Lending ratio: owner may borrow up to 30% of total documented revenue.
// In production this would be parameterizable and tied to a rolling window.
const LOAN_RATIO_PCT: i128 = 30;

#[contract]
pub struct SariLedgerContract;

#[contractimpl]
impl SariLedgerContract {
    /// One-time setup. Binds the contract to the sari-sari store owner's address.
    /// Must be called before any other method. Panics on re-initialization.
    pub fn initialize(env: Env, owner: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::TotalRevenue, &0_i128);
        env.storage().instance().set(&DataKey::SaleCount, &0_u32);
        env.storage().instance().set(&DataKey::LoanCount, &0_u32);
    }

    /// Owner records a restock: adds `quantity` units of `product_id` at
    /// unit cost `cost`. Weighted-average cost is recomputed so margin can
    /// be audited later. Only the store owner may call this.
    pub fn restock(env: Env, product_id: Symbol, quantity: u32, cost: i128) {
        Self::require_owner(&env);
        assert!(quantity > 0, "quantity must be positive");
        assert!(cost >= 0, "cost cannot be negative");

        let key = DataKey::Inventory(product_id);
        let current: InventoryItem = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(InventoryItem { quantity: 0, avg_cost: 0 });

        // Weighted-average unit cost across old + new stock.
        let old_value = current.avg_cost * (current.quantity as i128);
        let new_value = cost * (quantity as i128);
        let new_qty = current.quantity + quantity;
        let new_avg = (old_value + new_value) / (new_qty as i128);

        env.storage().persistent().set(
            &key,
            &InventoryItem { quantity: new_qty, avg_cost: new_avg },
        );
    }

    /// Owner records a sale of `quantity` units at `price` each.
    /// Decrements inventory, appends a Sale record, and bumps total revenue.
    /// Panics if product is unstocked or inventory is insufficient.
    pub fn record_sale(env: Env, product_id: Symbol, quantity: u32, price: i128) {
        Self::require_owner(&env);
        assert!(quantity > 0, "quantity must be positive");
        assert!(price >= 0, "price cannot be negative");

        // Decrement inventory.
        let inv_key = DataKey::Inventory(product_id.clone());
        let mut inventory: InventoryItem = env
            .storage()
            .persistent()
            .get(&inv_key)
            .expect("product does not exist");

        if inventory.quantity < quantity {
            panic!("insufficient inventory");
        }
        inventory.quantity -= quantity;
        env.storage().persistent().set(&inv_key, &inventory);

        // Append sale to ledger.
        let mut sale_count: u32 =
            env.storage().instance().get(&DataKey::SaleCount).unwrap_or(0);
        let sale = Sale {
            product_id,
            quantity,
            price,
            timestamp: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&DataKey::Sale(sale_count), &sale);
        sale_count += 1;
        env.storage().instance().set(&DataKey::SaleCount, &sale_count);

        // Bump running revenue — this is the number lenders underwrite against.
        let mut revenue: i128 =
            env.storage().instance().get(&DataKey::TotalRevenue).unwrap_or(0);
        revenue += price * (quantity as i128);
        env.storage().instance().set(&DataKey::TotalRevenue, &revenue);
    }

    /// Owner requests a loan. Eligibility = loan amount must be <=
    /// LOAN_RATIO_PCT% of total documented revenue. Returns the loan id.
    /// In production this would trigger a USDC transfer from a lending pool.
    pub fn request_loan(env: Env, amount: i128) -> u32 {
        Self::require_owner(&env);
        assert!(amount > 0, "loan amount must be positive");

        let revenue: i128 =
            env.storage().instance().get(&DataKey::TotalRevenue).unwrap_or(0);
        let max_loan = revenue * LOAN_RATIO_PCT / 100;

        if amount > max_loan {
            panic!("loan exceeds eligible amount");
        }

        let mut loan_count: u32 =
            env.storage().instance().get(&DataKey::LoanCount).unwrap_or(0);
        let loan = Loan { amount, repaid: 0, timestamp: env.ledger().timestamp() };
        env.storage().persistent().set(&DataKey::Loan(loan_count), &loan);

        let loan_id = loan_count;
        loan_count += 1;
        env.storage().instance().set(&DataKey::LoanCount, &loan_count);
        loan_id
    }

    /// Owner repays part or all of a loan. Increases the `repaid` field;
    /// loan is considered closed when repaid >= amount.
    pub fn repay_loan(env: Env, loan_id: u32, amount: i128) {
        Self::require_owner(&env);
        assert!(amount > 0, "repayment must be positive");

        let key = DataKey::Loan(loan_id);
        let mut loan: Loan =
            env.storage().persistent().get(&key).expect("loan does not exist");
        loan.repaid += amount;
        env.storage().persistent().set(&key, &loan);
    }

    /// Read-only view of an SKU's current inventory. Returns zero-valued
    /// item if product has never been stocked.
    pub fn get_inventory(env: Env, product_id: Symbol) -> InventoryItem {
        env.storage()
            .persistent()
            .get(&DataKey::Inventory(product_id))
            .unwrap_or(InventoryItem { quantity: 0, avg_cost: 0 })
    }

    /// Read-only total revenue — the figure that anchors loan eligibility.
    pub fn get_revenue(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalRevenue).unwrap_or(0)
    }

    /// Read-only view of a specific loan by id.
    pub fn get_loan(env: Env, loan_id: u32) -> Loan {
        env.storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .expect("loan does not exist")
    }

    // --- internal helpers ---------------------------------------------------

    fn require_owner(env: &Env) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("contract not initialized");
        owner.require_auth();
    }
}

#[cfg(test)]
mod test;