
impl Hash for ScopePtr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().0.hash(state)
    }
}

impl PartialEq for ScopePtr {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

これにより
関数は全て0のIDを共有するため
同じ名前で同じスコープなら同じ関数となり
古いものは上書きされる