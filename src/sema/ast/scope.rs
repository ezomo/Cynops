use super::*;
use crate::codegen::CodeGenSpace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeId(pub Vec<usize>);
impl ScopeId {
    pub fn root() -> Self {
        ScopeId(vec![])
    }
    pub fn child_of(&self, index: usize) -> Self {
        let mut new = self.0.clone();
        new.push(index);
        ScopeId(new)
    }

    pub fn id(&self) -> Vec<usize> {
        self.0.clone()
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
    pub fn get_variable(&self, ident: &Ident) -> Option<Type> {
        let mut scope = Some(Rc::clone(&self.current_scope));
        while let Some(s) = scope {
            if let Some(ty) = s.borrow().symbols.get(ident) {
                return Some(ty.clone());
            }
            scope = s.borrow().parent.as_ref().and_then(|p| p.upgrade());
        }
        None
    }

    pub fn get_scope(&self, name: &Ident) -> Rc<RefCell<ScopeNode>> {
        let mut scope = Some(Rc::clone(&self.current_scope));

        while let Some(s) = scope {
            if s.borrow().symbols.contains_key(name) {
                return s; // このスコープに定義されている
            }
            scope = s.borrow().parent.as_ref().and_then(|p| p.upgrade());
        }
        panic!("{} is undefined", name.to_string())
    }

    // 登録はcurrent_scopeに対して行う
    pub fn register_symbols(&mut self, name: Ident, ty: Type) {
        self.current_scope.borrow_mut().symbols.insert(name, ty);
    }

    pub fn register_function(&mut self, name: Ident, variants: Type) {
        self.root_scope.borrow_mut().symbols.insert(name, variants);
    }

    pub fn id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    pub fn scope_id(&self) -> String {
        self.current_scope
            .borrow()
            .id
            .id()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(".")
    }
}
