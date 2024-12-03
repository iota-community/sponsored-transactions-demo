
/// Module: sponsored_transactions_packages
module sponsored_transactions_packages::sponsored_transactions_packages {

use iota::event;
use iota::address;

    /// Subscriptions
    public struct Subscribed has copy, drop {
        id: address,
    }

    fun subscribe(ctx: &mut TxContext) {
        // get the address from the context
        let addr = ctx.sender();

        // emit the event
        event::emit(Subscribed { id: addr });
    }

}

