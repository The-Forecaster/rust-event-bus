pub mod event_bus {
    use std::{any::Any, collections::HashMap, io::Result};

    /// Structure for events
    ///
    /// Create a new event using the [Event::new] function. Create listener functions/data structures that use the type [Subscriber] and post events using an [EventBus]
    /// Different types of events will be tracked with names so keep your spelling and data type consistent (there will be a lot of messy conversion)
    pub struct Event {
        name: String,           // Track different events by string
        pub data: Box<dyn Any>, // Data can be anything, it could even be another boat!
    }

    /////////////////////////////////////////////////////////////////////////////
    // Type implementation
    /////////////////////////////////////////////////////////////////////////////
    impl Event {
        /// Creates a new event with the provided name and data
        pub fn new<S, D>(name: S, data: D) -> Self
        where
            S: Into<String>, // Allows use of both Strings and &strs as well as anything else people wanna put as a name
            D: 'static
        {
            Event {
                name: name.into(),
                data: Box::new(data),
            }
        }

        pub fn data<T: 'static>(&self) -> &T { // If this panics ur fucked
            let data = &self.data.downcast_ref::<T>().unwrap();

            return data;
        }
    }

    /// Trait for invoking a function when an [Event] is posted
    pub trait Subscriber {
        /// Function that will be called when
        fn call(&mut self, event: &Event) -> Result<()>;
    }

    /// Event Bus structure
    ///
    /// This will store the names of events and their corresponding [Subscriber] objects
    pub struct EventBus {
        event_subscribers: HashMap<String, Vec<Box<dyn Subscriber>>>,
    }

    /////////////////////////////////////////////////////////////////////////////
    // Type implementation
    /////////////////////////////////////////////////////////////////////////////
    impl EventBus {
        /// Creates a new [EventBus] with an empty subscriber map
        pub fn new() -> Self {
            EventBus {
                event_subscribers: HashMap::new(),
            }
        }

        /// Creates a new [EventBus] with the provided subscriber list
        pub fn from(event_subscribers: HashMap<String, Vec<Box<dyn Subscriber>>>) -> Self {
            EventBus { event_subscribers }
        }

        /// Adds the provided [Subscriber] to the subscriber list of the provided [String]
        pub fn subscribe<N, S>(&mut self, event_name: N, subscriber: S) -> Result<()>
        where
            N: Into<String>,
            S: Subscriber + 'static
        {
            let event = event_name.into();

            match self.event_subscribers.get(&event) {
                Some(_) => {
                    // If event is already registered to the map
                    self.event_subscribers
                        .get_mut(&event)
                        .unwrap()
                        .push(Box::new(subscriber));
                }
                None => {
                    // If event is not registered to map
                    self.event_subscribers
                        .insert(event.into(), vec![Box::new(subscriber)]);
                }
            }

            Ok(())
        }

        pub fn unsubscribe<N, S>(&mut self, event_name: N, subscriber: S) -> Result<()>
        where
            N: Into<String>,
            S: Subscriber + 'static,
        {
            // self.event_subscribers.get_mut(&event_name).unwrap().remove(self.event_subscribers.iter().position(|event_subscriber| event_subscriber.type_id() == subscriber.type_id()).unwrap());

            // Dumb if statements
            // TODO: Optimize this
            if let Some(subscribers) = self.event_subscribers.get_mut(&event_name.into()) {
                if let Some(index) =
                    subscribers
                        .iter()
                        .position(|event_subscriber| {
                            event_subscriber.type_id() == subscriber.type_id()
                        })
                {
                    subscribers.remove(index);
                }
            }

            Ok(())
        }

        /// Adds all [Subscriber] objects in the provided vec to the subscriber list of the provided [String]
        pub fn subscribe_all<N, S>(&mut self, event_name: &N, subscribers: Vec<S>) -> Result<()>
        where
            N: Into<String> + Clone,
            S: Subscriber + 'static
        {
            for subscriber in subscribers {
                self.subscribe(event_name.clone(), subscriber)
                    .expect(&format!(
                        "Error when subscribing to event: {}",
                        event_name.clone().into() // Kinda dumb but iteration sucks
                    ))
            }

            Ok(())
        }

        pub fn unsubscribe_all<N, S>(&mut self, event_name: &N, subscribers: Vec<S>) -> Result<()>
        where
            N: Into<String> + Clone,
            S: Subscriber + 'static
        {
            for subscriber in subscribers {
                self.unsubscribe(event_name.clone(), subscriber)
                    .expect(&format!(
                        "Error when unsubscribing from event: {}",
                        event_name.clone().into() // Kinda dumb but iteration sucks
                    ))
            }

            Ok(())
        }

        /// Posts an [Event] to all [Subscriber]s that are listening on that event name
        pub fn post(&mut self, event: Event)
        {
            if let Some(subscribers) = self.event_subscribers.get_mut(&event.name) {
                for subscriber in subscribers {
                    subscriber
                        .call(&event)
                        .expect(&format!("Error when posting event: {}", event.name));
                }
            }
        }
    }
}

fn main() {
    use event_bus::*;
    use std::{io::Result, thread, time::Duration};

    struct Ticker;

    impl Ticker {
        fn new() -> Self {
            Ticker {}
        }
    }

    impl Subscriber for Ticker {

        fn call(&mut self, event: &Event) -> Result<()> {
            println!("Tock!, {}", event.data::<i32>());

            Ok(())
        }
    }

    let event = Event::new("TickEvent", 32);

    let mut event_bus = EventBus::new();

    event_bus.subscribe("TickEvent", Ticker::new()).unwrap();

    event_bus.post(event);

    for i in 0..10 {
        event_bus.post(Event::new("TickEvent", i));
        thread::sleep(Duration::from_millis(100));
    }
}
