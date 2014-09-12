#![feature(tuple_indexing)]

use std::any::{Any, AnyRefExt};
use std::collections::HashMap;
use std::intrinsics::TypeId;
use std::sync::Mutex;

pub mod behaviors;

mod static_indices_vec;

/// State of the game.
pub struct State {
    entities: Mutex<static_indices_vec::StaticIndicesVec<EntityData>>,
    globals: EntityData,
}

/// Identifier for an entity.
/// All identifiers are only used once.
#[deriving(Clone, Show, PartialEq, Eq, Hash)]
pub struct EntityID(static_indices_vec::Index);

pub trait Property<T: Any> {
}

trait PropertyValue {
    fn get_value<'a>(&'a self, state: &'a State, entity: Option<Entity<'a>>) -> Box<Any>;
}

impl State {
    pub fn new() -> State {
        State {
            entities: Mutex::new(static_indices_vec::StaticIndicesVec::new()),
            globals: EntityData { properties: HashMap::new() },
        }
    }

    pub fn add<T: Clone + 'static, S: Send, P: Property<T> + 'static, V: behaviors::BehaviorFactory<S, T>>(&mut self, property: P, value: V) {
        let value = value.into_behavior();
        self.globals.properties.insert(TypeId::of::<P>(), Some(box value as Box<PropertyValue + 'static>));
    }

    pub fn get<T: Clone + 'static, P: Property<T> + 'static>(&self, property: P) -> Option<T> {
        self.globals.properties.find(&TypeId::of::<P>()).map(|val| val.as_ref().unwrap().get_value(self, None))
            .and_then(|val| { let val: &Any = &*val; val.downcast_ref::<T>().map(|e| e.clone()) })
    }

    pub fn entity(&self, entity: &EntityID) -> Option<Entity> {
        // FIXME: check if it exists
        Some(Entity {
            state: self,
            id: entity.0.clone()
        })
    }

    pub fn create_entity(&self) -> Entity {
        let mut entities = self.entities.lock();
        let new_id = entities.push(EntityData { properties: HashMap::new() });
        Entity {
            state: self,
            id: new_id,
        }
    }
}

struct EntityData {
    properties: HashMap<TypeId, Option<Box<PropertyValue + 'static>>>,
}

pub struct Entity<'s> {
    state: &'s State,
    id: static_indices_vec::Index,
}

impl EntityData {
    pub fn get<'a, T: Clone + 'static, P: Property<T> + 'static>(&'a self, property: P, state: &'a State, entity: Option<Entity<'a>>) -> Option<T> {
        self.properties.find(&TypeId::of::<P>()).map(|val| val.as_ref().unwrap().get_value(state, entity))
            .and_then(|val| { let val: &Any = &*val; val.downcast_ref::<T>().map(|e| e.clone()) })
    }
}

impl<'s> Entity<'s> {
    /// Returns the identifier of this entity
    pub fn get_id(&self) -> EntityID {
        EntityID(self.id.clone())
    }

    pub fn add<T: Clone + 'static, S: Send, P: Property<T> + 'static, V: behaviors::BehaviorFactory<S, T>>(&mut self, property: P, value: V) {
        let value = value.into_behavior();

        let mut entities = self.state.entities.lock();
        entities.get_mut(&self.id).expect("Internal error in FRP library").properties.insert(TypeId::of::<P>(), Some(box value as Box<PropertyValue + 'static>));
    }

    pub fn get<T: Clone + 'static, P: Property<T> + 'static>(&self, property: P) -> Option<T> {
        let to_call = {
            let mut entities = self.state.entities.lock();
            let mut entity = entities.get_mut(&self.id).expect("Internal error in FRP library");
            
            match entity.properties.find_mut(&TypeId::of::<P>()) {
                None => return None,
                Some(v) => v.take()
            }
        }.expect("Infinite recursive behaviors");

        let ret_value = {
            let value = to_call.get_value(self.state, Some(Entity { state: self.state, id: self.id.clone() }));
            let value: &Any = &*value;
            value.downcast_ref::<T>().map(|e| e.clone())
        };

        {
            let mut entities = self.state.entities.lock();
            let mut entity = entities.get_mut(&self.id).expect("Internal error in FRP library");
            
            match entity.properties.find_mut(&TypeId::of::<P>()) {
                None => fail!(),
                Some(v) => *v = Some(to_call)
            }
        }

        ret_value
    }
}
