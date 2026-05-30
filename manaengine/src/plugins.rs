use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    actions::{ContextItem, ContextPredicate, Listener},
    cards::AssignedBand,
};

type SemVer = [u8; 3];
struct RelationVec<K, V>(Vec<(K, V)>);

struct PluginPlugin {
    dependencies: Vec<(String, SemVer)>,
    version: SemVer,
    mincompat: SemVer,
    name: String,    
}

struct GamePlugin<'a> {
    global_predicate: RelationVec<String, ContextPredicate>,
    global_initialiser: RelationVec<String, ContextItem>,
    listeners: RelationVec<TypeId, Listener<'a>>
}

struct PluginRegistry<P: Clone> {
    registry: HashMap<String, P>,
}

impl<P: Clone> PluginRegistry<P> {
    fn getplugin<T>(&self, name: String) -> Option<T>
    where
        P: Into<T>,
    {
        self.registry
            .get(&name)
            .map(|p| <P as Into<T>>::into(p.clone()))
    }
}

struct BandPlugin<'g, C> {
    predicate: Vec<ContextPredicate>,
    constructor: fn(Vec<ContextItem>) -> Box<dyn AssignedBand<'g, C> + 'g>,
}

struct GameRegistry {
    registry: HashMap<TypeId, HashMap<String, Box<dyn Any>>>,
}

impl GameRegistry {
    fn insert<T>(&mut self, name: String, plugin: T) {
        let tid = TypeId::of::<T>();
        self.
    }
}
