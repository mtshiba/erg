use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use erg_common::dict::Dict;
use erg_common::levenshtein::get_similar_name;
use erg_common::shared::Shared;
use erg_common::Str;

use erg_parser::ast::VarName;

use crate::context::Context;
use crate::hir::HIR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModId(usize);

impl ModId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
    pub const fn builtin() -> Self {
        Self(0)
    }
    pub const fn main() -> Self {
        Self(1)
    }
}

#[derive(Debug, Clone)]
pub struct ModuleEntry {
    id: ModId, // builtin == 0, __main__ == 1
    pub hir: Option<HIR>,
    ctx: Rc<Context>,
}

impl fmt::Display for ModuleEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ModuleEntry(id = {}, name = {})",
            self.id.0, self.ctx.name
        )
    }
}

impl ModuleEntry {
    pub fn new(id: ModId, hir: Option<HIR>, ctx: Context) -> Self {
        Self {
            id,
            hir,
            ctx: Rc::new(ctx),
        }
    }

    pub fn builtin(ctx: Context) -> Self {
        Self {
            id: ModId::builtin(),
            hir: None,
            ctx: Rc::new(ctx),
        }
    }
}

#[derive(Debug, Default)]
pub struct ModuleCache {
    cache: Dict<VarName, ModuleEntry>,
    last_id: usize,
}

impl fmt::Display for ModuleCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModuleCache {}", self.cache)
    }
}

impl ModuleCache {
    pub fn new() -> Self {
        Self {
            cache: Dict::new(),
            last_id: 0,
        }
    }

    pub fn get<Q: Eq + Hash + ?Sized>(&self, name: &Q) -> Option<&ModuleEntry>
    where
        VarName: Borrow<Q>,
    {
        self.cache.get(name)
    }

    pub fn get_by_name(&self, __name__: &str) -> Option<(&VarName, &ModuleEntry)> {
        self.cache
            .iter()
            .find(|(_, ent)| &ent.ctx.name[..] == __name__)
    }

    pub fn get_mut<Q: Eq + Hash + ?Sized>(&mut self, name: &Q) -> Option<&mut ModuleEntry>
    where
        VarName: Borrow<Q>,
    {
        self.cache.get_mut(name)
    }

    pub fn register(&mut self, name: VarName, hir: Option<HIR>, ctx: Context) {
        self.last_id += 1;
        let id = ModId::new(self.last_id);
        let entry = ModuleEntry::new(id, hir, ctx);
        self.cache.insert(name, entry);
    }

    pub fn register_alias(&mut self, name: VarName, entry: &ModuleEntry) {
        self.cache.insert(name, entry.clone());
    }

    pub fn remove<Q: Eq + Hash + ?Sized>(&mut self, name: &Q) -> Option<ModuleEntry>
    where
        VarName: Borrow<Q>,
    {
        self.cache.remove(name)
    }

    pub fn remove_by_id(&mut self, id: ModId) -> Option<ModuleEntry> {
        if let Some(name) = self.cache.iter().find_map(|(name, ent)| {
            if ent.id == id {
                Some(name.clone())
            } else {
                None
            }
        }) {
            self.remove(&name)
        } else {
            None
        }
    }

    pub fn get_similar_name(&self, name: &str) -> Option<Str> {
        get_similar_name(self.cache.iter().map(|(v, _)| &v.inspect()[..]), name).map(Str::rc)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SharedModuleCache(Shared<ModuleCache>);

impl fmt::Display for SharedModuleCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shared{}", self.0)
    }
}

impl SharedModuleCache {
    pub fn new() -> Self {
        let self_ = Self(Shared::new(ModuleCache::new()));
        Context::init_builtins(&self_);
        self_
    }

    pub fn get<Q: Eq + Hash + ?Sized>(&self, name: &Q) -> Option<&ModuleEntry>
    where
        VarName: Borrow<Q>,
    {
        let ref_ = unsafe { self.0.as_ptr().as_ref().unwrap() };
        ref_.get(name)
    }

    pub fn get_by_name(&self, __name__: &str) -> Option<(&VarName, &ModuleEntry)> {
        let ref_ = unsafe { self.0.as_ptr().as_ref().unwrap() };
        ref_.get_by_name(__name__)
    }

    pub fn get_mut<Q: Eq + Hash + ?Sized>(&self, name: &Q) -> Option<&mut ModuleEntry>
    where
        VarName: Borrow<Q>,
    {
        let ref_ = unsafe { self.0.as_ptr().as_mut().unwrap() };
        ref_.get_mut(name)
    }

    pub fn get_ctx<Q: Eq + Hash + ?Sized>(&self, name: &Q) -> Option<Rc<Context>>
    where
        VarName: Borrow<Q>,
    {
        self.0.borrow().get(name).map(|entry| entry.ctx.clone())
    }

    pub fn ref_ctx<Q: Eq + Hash + ?Sized>(&self, name: &Q) -> Option<&Context>
    where
        VarName: Borrow<Q>,
    {
        let ref_ = unsafe { self.0.as_ptr().as_ref().unwrap() };
        ref_.get(name).map(|entry| entry.ctx.as_ref())
    }

    pub fn register(&self, name: VarName, hir: Option<HIR>, ctx: Context) {
        self.0.borrow_mut().register(name, hir, ctx);
    }

    pub fn register_alias(&self, name: VarName, entry: &ModuleEntry) {
        self.0.borrow_mut().register_alias(name, entry);
    }

    pub fn remove<Q: Eq + Hash + ?Sized>(&self, name: &Q) -> Option<ModuleEntry>
    where
        VarName: Borrow<Q>,
    {
        self.0.borrow_mut().remove(name)
    }

    pub fn remove_by_id(&self, id: ModId) -> Option<ModuleEntry> {
        self.0.borrow_mut().remove_by_id(id)
    }

    pub fn get_similar_name(&self, name: &str) -> Option<Str> {
        self.0.borrow().get_similar_name(name)
    }
}