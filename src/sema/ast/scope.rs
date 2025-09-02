use super::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct ScopeNode {
    pub symbols: HashMap<Ident, Type>,
    pub parent: Option<Weak<RefCell<ScopeNode>>>,
    pub children: Vec<Rc<RefCell<ScopeNode>>>,
}

#[derive(Debug)]
pub struct Session {
    pub root_scope: Rc<RefCell<ScopeNode>>,
    pub current_scope: Rc<RefCell<ScopeNode>>,
}

impl ScopeNode {
    pub fn new(parent: Option<Rc<RefCell<ScopeNode>>>) -> Rc<RefCell<Self>> {
        let weak_parent = parent.as_ref().map(|p| Rc::downgrade(p));
        Rc::new(RefCell::new(ScopeNode {
            symbols: HashMap::new(),
            parent: weak_parent,
            children: Vec::new(),
        }))
    }
}

impl Session {
    pub fn new() -> Self {
        let root = ScopeNode::new(None);
        Self {
            root_scope: Rc::clone(&root),
            current_scope: root,
        }
    }

    // 新しいスコープを作って移動
    pub fn push_scope(&mut self) {
        let new_scope = ScopeNode::new(Some(Rc::clone(&self.current_scope)));
        self.current_scope
            .borrow_mut()
            .children
            .push(Rc::clone(&new_scope));
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
    pub fn get_variable(&self, name: &Ident) -> Option<Type> {
        let mut scope = Some(Rc::clone(&self.current_scope));
        while let Some(s) = scope {
            if let Some(ty) = s.borrow().symbols.get(name) {
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
}
