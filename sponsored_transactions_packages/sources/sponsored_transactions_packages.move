/// Module: sponsored_transactions_packages
module sponsored_transactions_packages::sponsored_transactions_packages {

    use iota::event;
    use iota::linked_table::{Self, LinkedTable};
    use iota::package::Publisher;
    use std::string::{Self, String};

    const ENotAuthorized: u64 = 0;

    /// Enum representing subscription types
    public enum SubscriptionType has copy, drop, store {
        Music,
        Movies,
        News,
    }

    public struct Content has key,store {
        id: UID,
        content_type: SubscriptionType,
        content: String,
    }

    /// Event for subscriptions
    public struct Subscribed has copy, drop {
        id: address,
        subscription_type: SubscriptionType,
    }


    /// The OTW type for the Subscription Manager
    public struct SPONSORED_TRANSACTIONS_PACKAGES has drop {}

    /// Subscription manager struct encapsulating subscription and trial tables
    public struct SubscriptionManager has key, store {
        id: UID,
        subscriptions: LinkedTable<address, SubscriptionType>,
        trials: LinkedTable<address, SubscriptionType>,
    }

    /// function init for SubscriptionManager
    fun init( otw: SPONSORED_TRANSACTIONS_PACKAGES, ctx: &mut TxContext) {
        let manager = SubscriptionManager {
            id: object::new(ctx),
            subscriptions: linked_table::new<address, SubscriptionType>(ctx),
            trials: linked_table::new<address, SubscriptionType>(ctx),
        };

        let publisher: Publisher = iota::package::claim(otw, ctx);


        // Share the manager
        transfer::public_share_object( manager);

        // transfer the publisher to the sender
        transfer::public_transfer( publisher, ctx.sender());
    }

    /// Allows a user to subscribe to a specific type of content
    entry fun subscribe(subscription_type: String, manager: &mut SubscriptionManager, ctx: &mut TxContext) {

        let mut sub_type = SubscriptionType::News;
        // get the type of subscription
        if (subscription_type.as_bytes() == b"Music") {
            sub_type = SubscriptionType::Music;
        } else if (subscription_type.as_bytes() == b"Movies") {
            sub_type = SubscriptionType::Movies;
        } else if (subscription_type.as_bytes() == b"News") {
            sub_type = SubscriptionType::News;
        } else {
            let sub_type = SubscriptionType::News;
        };

        // Get the address from the context
        let addr = ctx.sender();

        // Add the subscription to the subscription table
        manager.subscriptions.push_back(ctx.sender(), sub_type);

        // Emit the subscription event
        event::emit(Subscribed { id: addr, subscription_type: sub_type });
    }

    entry fun free_trial(subscription_type: String, manager: &mut SubscriptionManager, ctx: &mut TxContext) {

        let mut sub_type = SubscriptionType::News;
        // get the type of subscription
        if (subscription_type.as_bytes() == b"Music") {
            sub_type = SubscriptionType::Music;
        } else if (subscription_type.as_bytes() == b"Movies") {
            sub_type = SubscriptionType::Movies;
        } else if (subscription_type.as_bytes() == b"News") {
            sub_type = SubscriptionType::News;
        } else {
            sub_type = SubscriptionType::News;
        };

        // Get the address from the context
        let addr = ctx.sender();

        // Add the subscription to the trial table
        manager.trials.push_back(ctx.sender(), sub_type);

        // Emit the subscription event
        event::emit(Subscribed { id: addr, subscription_type: sub_type});

    }

    /// publish content to the subscribers and trials
    /// This function is only callable by the publisher, it loops through the subscriptions and trials
    /// and sends the content to the subscribers and trials
    entry fun publish(manager: &mut SubscriptionManager, publisher: &Publisher, ctx: &mut TxContext) {
        
        assert!(publisher.from_module<SubscriptionManager>(), ENotAuthorized);


        let mut first_sub = manager.subscriptions.front();
        let mut first_trial = manager.trials.front();

        // // Loop through the subscriptions and send the content
        // for (addr, type) in subscriptions {
        //     let content = Content {
        //         id: object::new(ctx),
        //         content_type: type,
        //         content: String::utf8(b"This is the content"),
        //     };
        //     transfer::send(publisher, addr, content);
        // }

        // // Loop through the trials and send the content
        while (option::is_some(first_sub)) {
            

            let k = *option::borrow(first_sub); // option::borrow - returns an immutable reference to the value inside the Option
            let v = linked_table::borrow(& manager.subscriptions, k);


            let content = Content {
                id: object::new(ctx),
                content_type: *v,
                content: string::utf8(b"This is the content"),
            };
            transfer::transfer(content, k);
            first_sub = manager.subscriptions.next(k);
        };

        while (option::is_some(first_trial)) {
            let k = *option::borrow(first_trial); // option::borrow - returns an immutable reference to the value inside the Option
            let v = linked_table::borrow(& manager.trials, k);

            let content = Content {
                id: object::new(ctx),
                content_type: *v,
                content: string::utf8(b"This is the content"),
            };
            transfer::transfer(content, k);
            first_trial = manager.trials.next(k);
        };

        

    }
}