
//mod block;
mod chain {
    pub mod chain;
    pub mod block {
        pub mod block;
    }
}

use crate::chain::chain::chain::Chain as Chain;

fn main() {
    let my_chain = Chain::new(String::from("my_chain"));

    println!("Hello, world!");
}
