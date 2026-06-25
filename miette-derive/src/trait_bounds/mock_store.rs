#![allow(dead_code)]
use syn::{punctuated::Punctuated, Generics, PredicateType, Token, WhereClause, WherePredicate};

// Mock for when perfect-derive is not enabled,
// this should be completely optimized away and enables
// easily switching on/off the perfect-derive feature without
// needing to modify any other code.
pub struct TypeParamBoundStore;

impl TypeParamBoundStore {
    pub fn new(_: &Generics) -> Self {
        Self
    }

    pub fn add_predicate(&mut self, _: PredicateType) {}

    pub fn add_where_predicate(&mut self, _: WherePredicate) {}

    pub fn extend_where_predicates(&mut self, _: Punctuated<WherePredicate, Token![,]>) {}

    pub fn add_to_where_clause(&self, where_clause: Option<&WhereClause>) -> Option<WhereClause> {
        where_clause.cloned()
    }
}
