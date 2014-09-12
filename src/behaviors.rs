use std::any::Any;
use std::sync::Mutex;
use {Entity, State};

/// Builder for a `Behavior`.
pub trait BehaviorFactory<S, T> {
    fn into_behavior(self) -> Behavior<S, T>;
}

/// Prototype for a `Behavior` with a constant value.
pub struct Constant<T>(pub T);

impl<T> BehaviorFactory<(), T> for Constant<T> {
    fn into_behavior(self) -> Behavior<(), T> {
        let Constant(value) = self;
        Behavior(ConstantBehavior_(value))
    }
}

/// Prototype for a `Behavior` whose value is an alias of something else.
pub struct Alias<T>(pub fn(&State, Option<Entity>) -> T);

impl<T> BehaviorFactory<(), T> for Alias<T> {
    fn into_behavior(self) -> Behavior<(), T> {
        let Alias(value) = self;
        Behavior(AliasBehavior_(value))
    }
}

/// Prototype for a `Behavior` whose value stores something.
pub struct Storage<S, T>(pub S, pub fn(&mut S, &State, Option<Entity>) -> T);

impl<S: Send, T> BehaviorFactory<S, T> for Storage<S, T> {
    fn into_behavior(self) -> Behavior<S, T> {
        let Storage(initial, function) = self;
        Behavior(StorageBehavior_(Mutex::new(Some(initial)), function))
    }
}

/// You can't create a `Behavior` youself. Instead you must use another implementor
///  of `BehaviorFactory`.
#[doc(hidden)]
pub struct Behavior<S, T>(Behavior_<S, T>);

enum Behavior_<S, T> {
    ConstantBehavior_(T),
    AliasBehavior_(fn(&State, Option<Entity>) -> T),
    StorageBehavior_(Mutex<Option<S>>, fn(&mut S, &State, Option<Entity>) -> T),
}

impl<S: Send, T: Clone + 'static> super::PropertyValue for Behavior<S, T> {
    fn get_value<'a>(&'a self, state: &'a State, entity: Option<Entity<'a>>) -> Box<Any> {
        match self {
            &Behavior(ConstantBehavior_(ref val)) => box val.clone() as Box<Any>,
            &Behavior(AliasBehavior_(ref val)) => {
                let val: T = (*val)(state, entity);
                box val as Box<Any>
            },
            &Behavior(StorageBehavior_(ref current, ref function)) => {
                let mut current_taken = current.lock().deref_mut().take()
                    .expect("Infinite recursive behaviors");
                let result = (*function)(&mut current_taken, state, entity);
                (*current.lock()) = Some(current_taken);
                box result as Box<Any>
            }
        }
    }
}
