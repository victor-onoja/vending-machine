#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use stylus_sdk::alloy_primitives::{Address, U256};
use stylus_sdk::prelude::*;
use stylus_sdk::{block, console};

sol_storage! {
    #[entrypoint]
    pub struct VendingMachine {
        mapping(address => uint256) cupcake_balances;
        mapping(address => uint256) cupcake_distribution_times;
    }
}

#[public]
impl VendingMachine {
    pub fn give_cupcake_to(&mut self, user_address: Address) -> bool {
        // Burn gas via repeated storage reads — these are real HostIO boundary
        // crossings that stylus-trace can see and measure.
        // Each .get() call hits storage_cache (warm) or storage_load (cold) HostIO.
        let mut acc = U256::ZERO;
        for _ in 0..20 {
            acc = acc.wrapping_add(self.cupcake_balances.get(user_address));
        }
        // Use acc so the compiler cannot eliminate the loop
        if acc == U256::MAX {
            return false;
        }
        let last_distribution = self.cupcake_distribution_times.get(user_address);
        let next_available = last_distribution + U256::from(5);
        let current_time = U256::from(block::timestamp());

        if next_available <= current_time {
            let mut balance_accessor = self.cupcake_balances.setter(user_address);
            let balance = balance_accessor.get() + U256::from(1);
            balance_accessor.set(balance);

            let mut time_accessor = self.cupcake_distribution_times.setter(user_address);
            time_accessor.set(current_time);
            true
        } else {
            console!(
                "HTTP 429: Too Many Cupcakes (you must wait at least 5 seconds between cupcakes)"
            );
            false
        }
    }

    pub fn get_cupcake_balance_for(&self, user_address: Address) -> U256 {
        self.cupcake_balances.get(user_address)
    }
}
