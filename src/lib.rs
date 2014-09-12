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

/// Trait that must be implemented by objects that serve as properties.
pub trait Property<T: Any> {}

/// 
trait PropertyValue {
    fn get_value<'a>(&'a self, state: &'a State, entity: Option<Entity<'a>>) -> Box<Any>;
}

impl State {
    /// Builds a new empty state.
    pub fn new() -> State {
        State {
            entities: Mutex::new(static_indices_vec::StaticIndicesVec::new()),
            globals: EntityData { properties: HashMap::new() },
        }
    }

    /// Adds a global property to the state.
    pub fn add<T: Clone + 'static, S: Send, P: Property<T> + 'static, V: behaviors::BehaviorFactory<S, T>>(&mut self, property: P, value: V) {
        let value = value.into_behavior();
        self.globals.properties.insert(TypeId::of::<P>(), Some(box value as Box<PropertyValue + 'static>));
    }

    /// Returns the value of a global property.
    /// 
    /// Returns `None` if the property doesn't exist.
    pub fn get<T: Clone + 'static, P: Property<T> + 'static>(&self, property: P) -> Option<T> {
        self.globals.properties.find(&TypeId::of::<P>()).map(|val| val.as_ref().unwrap().get_value(self, None))
            .and_then(|val| { let val: &Any = &*val; val.downcast_ref::<T>().map(|e| e.clone()) })
    }

    /// Builds a handle for an existing entity.
    pub fn entity(&self, entity: &EntityID) -> Option<Entity> {
        // FIXME: check if it exists
        Some(Entity {
            state: self,
            id: entity.0.clone()
        })
    }

    /// Builds a new entity and returns a handle to it.
    pub fn create_entity(&self) -> Entity {
        let mut entities = self.entities.lock();
        let new_id = entities.push(EntityData { properties: HashMap::new() });
        Entity {
            state: self,
            id: new_id,
        }
    }
}

/// Data storage for an entity.
struct EntityData {
    properties: HashMap<TypeId, Option<Box<PropertyValue + 'static>>>,
}

/// Handle to an entity in the state.
pub struct Entity<'s> {
    state: &'s State,
    id: static_indices_vec::Index,
}

impl<'s> Entity<'s> {
    /// Returns the identifier of this entity
    pub fn get_id(&self) -> EntityID {
        EntityID(self.id.clone())
    }

    /// Adds a property to this entity.
    pub fn add<T: Clone + 'static, S: Send, P: Property<T> + 'static, V: behaviors::BehaviorFactory<S, T>>(&mut self, property: P, value: V) {
        let value = value.into_behavior();

        let mut entities = self.state.entities.lock();
        entities.get_mut(&self.id).expect("Internal error in FRP library").properties.insert(TypeId::of::<P>(), Some(box value as Box<PropertyValue + 'static>));
    }

    /// Returns the value of a property of this entity.
    /// 
    /// Returns `None` if the property doesn't exist.
    pub fn get<T: Clone + 'static, P: Property<T> + 'static>(&self, property: P) -> Option<T> {
        // taking the Box<PropertyValue> from the entity
        //  so that the mutex is no longer locked when we invoke it
        let to_call = {
            let mut entities = self.state.entities.lock();
            let mut entity = entities.get_mut(&self.id).expect("Internal error in FRP library");
            
            match entity.properties.find_mut(&TypeId::of::<P>()) {
                None => return None,
                Some(v) => v.take()
            }
        }.expect("Infinite recursive behaviors");

        // invoking the PropertyValue
        let ret_value = {
            let value = to_call.get_value(self.state, Some(Entity { state: self.state, id: self.id.clone() }));
            let value: &Any = &*value;
            value.downcast_ref::<T>().map(|e| e.clone())
        };

        // storing the PropertyValue back in the entity
        {
            let mut entities = self.state.entities.lock();
            let mut entity = entities.get_mut(&self.id).expect("Internal error in FRP library");
            
            match entity.properties.find_mut(&TypeId::of::<P>()) {
                None => fail!(),
                Some(v) => *v = Some(to_call)
            }
        }

        // returning the value
        ret_value
    }
}
