use std::any::{Any, AnyRefExt};
use std::collections::HashMap;
use std::intrinsics::TypeId;

pub mod behaviors;

mod static_indices_vec;

/// State of the game.
pub struct State {
    entities: static_indices_vec::StaticIndicesVec<EntityData>,
    globals: EntityData,
}

pub trait Property<T: Any> {
}

trait PropertyValue {
    fn get_value<'a>(&'a self, state: &'a State) -> Box<Any>;
}

impl State {
    pub fn new() -> State {
        State {
            entities: static_indices_vec::StaticIndicesVec::new(),
            globals: EntityData { properties: HashMap::new() },
        }
    }

    pub fn add<T: Clone + 'static, S: Send, P: Property<T> + 'static, V: behaviors::BehaviorFactory<S, T>>(&mut self, property: P, value: V) {
        let value = value.into_behavior();
        self.globals.properties.insert(TypeId::of::<P>(), box value);
    }

    pub fn get<T: Clone + 'static, P: Property<T> + 'static>(&self, property: P) -> Option<T> {
        self.globals.properties.find(&TypeId::of::<P>()).map(|val| val.get_value(self))
            .and_then(|val| { let val: &Any = &*val; val.downcast_ref::<T>().map(|e| e.clone()) })
    }

    pub fn create_entity(&self) -> Entity {
        unimplemented!()
    }
}

struct EntityData {
    properties: HashMap<TypeId, Box<PropertyValue + 'static>>,
}

pub struct Entity<'s> {
    state: &'s State,
}

impl<'s> Entity<'s> {
}
