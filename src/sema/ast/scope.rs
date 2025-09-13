use super::*;
use crate::codegen::CodeGenSpace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Symbol {
    pub ident: Ident,
    pub scope: ScopePtr, // どこのスコープで定義されたか
}

impl Symbol {
    pub fn new(name: Ident, scope: ScopePtr) -> Self {
        Symbol {
            ident: name,
            scope: scope,
        }
    }

    // 変数検索（親も遡る）　２箇所で同じようなものがあるので良くない
    pub fn get_type(&self) -> Option<Type> {
        let mut scope = self.scope.get_scope(); // Weak -> Rc
        while let Some(s) = scope {
            if let Some(ty) = s.borrow().symbols.get(&self.ident) {
                return Some(ty.clone());
            }
            scope = s.borrow().parent.as_ref().and_then(|p| p.upgrade());
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct ScopePtr {
    pub ptr: Weak<RefCell<ScopeNode>>,
}
impl ScopePtr {
    pub fn new(ptr: Weak<RefCell<ScopeNode>>) -> Self {
        ScopePtr { ptr }
    }

    pub fn get_scope(&self) -> Option<Rc<RefCell<ScopeNode>>> {
        self.ptr.upgrade()
    }
}

impl Hash for ScopePtr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state);
    }
}

impl PartialEq for ScopePtr {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.upgrade().unwrap().borrow().id == other.ptr.upgrade().unwrap().borrow().id
    }
}

impl Eq for ScopePtr {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeId(pub Vec<usize>);
impl ScopeId {
    pub fn root() -> Self {
        ScopeId(vec![0])
    }
    pub fn child_of(&self, index: usize) -> Self {
        let mut new = self.0.clone();
        new.push(index);
        ScopeId(new)
    }

    pub fn id_vec(&self) -> Vec<usize> {
        self.0.clone()
    }

    pub fn id_string(&self) -> String {
        self.id_vec()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(".")
    }
}

#[derive(Debug)]
pub struct ScopeNode {
    pub id: ScopeId,
    pub codege_space: CodeGenSpace,
    pub symbols: HashMap<Ident, Type>,
    pub parent: Option<Weak<RefCell<ScopeNode>>>,
    pub children: Vec<Rc<RefCell<ScopeNode>>>,
}
impl ScopeNode {
    pub fn new(parent: Option<Rc<RefCell<ScopeNode>>>) -> Rc<RefCell<Self>> {
        let id = if let Some(ref p) = parent {
            let parent_id = &p.borrow().id;
            let index = p.borrow().children.len();
            parent_id.child_of(index)
        } else {
            ScopeId::root()
        };

        Rc::new(RefCell::new(ScopeNode {
            id,
            codege_space: CodeGenSpace::new(),
            symbols: HashMap::new(),
            parent: parent.as_ref().map(|p| Rc::downgrade(p)),
            children: Vec::new(),
        }))
    }

    pub fn add_child(parent: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        let child = ScopeNode::new(Some(Rc::clone(parent)));
        parent.borrow_mut().children.push(Rc::clone(&child));
        child
    }
}
#[derive(Debug)]
pub struct Session {
    pub root_scope: Rc<RefCell<ScopeNode>>,
    pub current_scope: Rc<RefCell<ScopeNode>>,
    pub id: usize,
}

impl Session {
    pub fn new() -> Self {
        let root = ScopeNode::new(None);
        Self {
            root_scope: Rc::clone(&root),
            current_scope: root,
            id: 0,
        }
    }

    pub fn current_scope(&self) -> ScopePtr {
        ScopePtr::new(Rc::downgrade(&self.current_scope))
    }

    pub fn push_scope(&mut self) {
        let new_scope = ScopeNode::add_child(&self.current_scope);
        self.current_scope = new_scope;
    }

    // 親スコープに戻る
    pub fn pop_scope(&mut self) {
        // borrow を先に取得して即座に clone する
        let parent_weak = self.current_scope.borrow().parent.clone();

        if let Some(parent_weak) = parent_weak {
            if let Some(parent) = parent_weak.upgrade() {
                self.current_scope = parent;
            }
        }
    }
    // 変数検索（親も遡る）

    pub fn get_ident_scope(&self, name: &Ident) -> ScopePtr {
        let mut scope = Some(Rc::clone(&self.current_scope));

        while let Some(s) = scope {
            if s.borrow().symbols.contains_key(name) {
                return ScopePtr::new(Rc::downgrade(&s)); // このスコープに定義されている
            }
            scope = s.borrow().parent.as_ref().and_then(|p| p.upgrade());
        }
        panic!("{} is undefined", name.to_string())
    }

    pub fn register_function(&mut self, name: Ident, variants: Type) {
        self.root_scope.borrow_mut().symbols.insert(name, variants);
    }

    pub fn id(&mut self) -> usize {
        self.id += 1;
        self.id
    }
}

pub trait ScopeNodes {
    fn get_type(&self, ident: &Ident) -> Option<Type>;
    fn register_symbols(&mut self, name: Ident, ty: Type);
}

impl ScopeNodes for ScopePtr {
    fn get_type(&self, ident: &Ident) -> Option<Type> {
        self.ptr.upgrade().unwrap().borrow().get_type(ident)
    }
    fn register_symbols(&mut self, name: Ident, ty: Type) {
        self.get_scope()
            .unwrap()
            .borrow_mut()
            .register_symbols(name, ty);
    }
}
impl ScopeNodes for Session {
    fn get_type(&self, ident: &Ident) -> Option<Type> {
        self.current_scope.borrow().get_type(ident)
    }
    fn register_symbols(&mut self, name: Ident, ty: Type) {
        self.current_scope.borrow_mut().register_symbols(name, ty);
    }
}
impl ScopeNodes for ScopeNode {
    fn get_type(&self, ident: &Ident) -> Option<Type> {
        if let Some(ty) = self.symbols.get(ident) {
            return Some(ty.clone());
        }
        let mut scope = self.parent.as_ref().and_then(|p| p.upgrade());
        while let Some(s) = scope {
            let borrowed = s.borrow();
            if let Some(ty) = borrowed.symbols.get(ident) {
                return Some(ty.clone());
            }
            scope = s.borrow().parent.as_ref().and_then(|p| p.upgrade());
        }
        None
    }

    fn register_symbols(&mut self, name: Ident, ty: Type) {
        self.symbols.insert(name, ty);
    }
}
