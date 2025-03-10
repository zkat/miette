use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter::once,
};

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, AssocType, BoundLifetimes,
    GenericArgument, GenericParam, Generics, ParenthesizedGenericArguments, PathArguments,
    PredicateType, ReturnType, Token, Type, TypeArray, TypeGroup, TypeParamBound, TypeParen,
    TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple, WhereClause, WherePredicate,
};

// Potential improvement, although idk if this actually ends up
// mattering is to switch this to something like FxHashMap like the rustc compiler uses internally
pub struct TypeParamBoundStore(HashMap<(Option<BoundLifetimes>, Type), HashSet<TypeParamBound>>);

impl TypeParamBoundStore {
    /// Creates a new TraitBoundStore, filling it with some generics which are used to heuristically remove trivial bounds.
    ///
    /// Note that it is essential that all relevant generics are actually passed here, since if they aren't bounds which are required might be heuristically removed.
    pub fn new(generics: &Generics) -> Self {
        let hash_map = generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Type(ty) => Some(ty),
                _ => None,
            })
            .map(|param| {
                let ident = &param.ident;
                Type::Path(syn::parse_quote!(#ident))
            })
            .map(|ty| ((None, ty), Default::default()))
            .collect::<HashMap<_, _>>();

        Self(hash_map)
    }

    /// Checks heuristically if `type` is using any generic type inside it.
    ///
    /// This is guaranteed to never false-negative but might
    /// false-positive if checking exhaustively would be expensive or
    /// an unexpected case is encountered which this can't handle.
    ///
    /// # Returns
    /// Option with a simplified type if determined to be dependant, none otherwise
    fn generic_usage_heuristics(&self, mut r#type: Type) -> Option<Type> {
        // in theory we could skip all this logic and just allow trivial bounds but that would add redundant trait bounds
        // to the derived impl - would be another choice to make. I choose to filter as much as possible so that we don't
        // introduce unneccessary bounds.

        // this reduces the type down as much as possible to remove unneeded groups.
        let original_type = loop {
            match r#type {
                Type::Paren(TypeParen { elem, .. }) => r#type = *elem,
                Type::Group(TypeGroup { elem, .. }) => r#type = *elem,
                x => break x,
            }
        };

        let mut depends_on_generic = false;

        // max depth to check, after which we'll just add the (maybe redundant) bound anyways.
        // this is a tradeoff between filtering speed and compiler speed so I'll keep it
        // reasonably low for now, since I assume the compiler is better optimized for more complex
        // checks.
        let max_depth = 8;

        let mut to_check_queue: VecDeque<(&Type, usize)> = VecDeque::new();
        to_check_queue.push_back((&original_type, 0));

        while !depends_on_generic {
            // this needs to be like this cuz if-let-chains aren't supported yet
            let Some((elem, current_depth)) = to_check_queue.pop_front() else {
                break;
            };

            // if we exceed the max depth we just assume it depends on the generic and let the compiler check it
            if current_depth > max_depth {
                depends_on_generic = true;
                break;
            }

            // the map contains types that we know depend on generics so we can just short circuit
            //
            // this is also the "bottom" check since we add the generics themselves to the map when
            // constructing self
            if self.0.contains_key(&(None, elem.clone())) {
                depends_on_generic = true;
                break;
            }

            // basically go through the type and add all referenced types inside it to the check queue
            match elem {
                Type::Group(_) => unreachable!("This is unwrapped above"),
                Type::Paren(_) => unreachable!("This is unwrapped above"),
                // function pointer's can never implement the required trait bounds anyways so we just accept the errors
                Type::BareFn(_) => return None,
                // impl trait types aren't allowed from struct/enum definitions anyways so we can just ignore them
                Type::ImplTrait(_) => return None,
                // infered types aren't allowed either
                Type::Infer(_) => return None,
                // macros are opaque to us and i don't really know how to properly implement this.
                // we could in theory I think introduce a type alias and use that instead but honestly
                // type macros are such a niche usecase especially in combination with a generic,
                // I would say we should just recommend to implement
                // the trait manually, as such we just accept the error if any occurs (this still allows using macros when they
                // return concrete types which don't depend on any generic or when the generic doesn't affect the
                // required trait implementation)
                Type::Macro(_) => return None,
                // trait objects which depend on a generic inside them seem like very much a hassle to implement so i'll ignore
                // them for now, if the need arises we could support that in a future pr maybe?
                //
                // this again doesn't restrict the usage of trait objects which implement the required traits regardless of the generics.
                Type::TraitObject(_) => return None,
                // Well never is never and never never.
                Type::Never(_) => return None,
                Type::Array(TypeArray { elem, .. })
                | Type::Ptr(TypePtr { elem, .. })
                | Type::Reference(TypeReference { elem, .. })
                | Type::Slice(TypeSlice { elem, .. }) => {
                    to_check_queue.push_back((&**elem, current_depth + 1));
                }
                Type::Path(TypePath { qself, path }) => {
                    if let Some(qself) = qself {
                        to_check_queue.push_back((&qself.ty, current_depth + 1));
                    }

                    for segment in &path.segments {
                        match &segment.arguments {
                            PathArguments::None => {}
                            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                args,
                                ..
                            }) => {
                                for argument in args {
                                    match argument {
                                        GenericArgument::Type(ty)
                                        | GenericArgument::AssocType(AssocType { ty, .. }) => {
                                            to_check_queue.push_back((ty, current_depth + 1));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            PathArguments::Parenthesized(ParenthesizedGenericArguments {
                                inputs,
                                output,
                                ..
                            }) => {
                                for inp in inputs {
                                    to_check_queue.push_back((inp, current_depth + 1));
                                }

                                if let ReturnType::Type(_, ty) = output {
                                    to_check_queue.push_back((ty, current_depth + 1));
                                }
                            }
                        }
                    }
                }
                Type::Tuple(TypeTuple { elems, .. }) => {
                    for elem in elems {
                        to_check_queue.push_back((elem, current_depth + 1));
                    }
                }
                // we can't really handle verbatim so we just assume it depends on the generics
                Type::Verbatim(_) => depends_on_generic = true,
                _ => depends_on_generic = true,
            }
        }

        depends_on_generic.then_some(original_type)
    }

    pub fn add_predicate(
        &mut self,
        PredicateType {
            lifetimes,
            bounded_ty,
            colon_token: _,
            bounds,
        }: PredicateType,
    ) {
        let Some(bounded_ty) = self.generic_usage_heuristics(bounded_ty) else {
            return;
        };

        self.0
            .entry((lifetimes, bounded_ty))
            .or_default()
            .extend(bounds);
    }

    // Since syn for some reason doesn't implement `Parse` for `PredicateType`
    // this method is meant for ease of use with `syn::parse_quote!`.
    pub fn add_where_predicate(&mut self, predicate: WherePredicate) {
        let WherePredicate::Type(ty) = predicate else {
            unimplemented!("Only type predicates are supported");
        };

        self.add_predicate(ty);
    }

    pub fn extend_where_predicates(&mut self, predicates: Punctuated<WherePredicate, Token![,]>) {
        for predicate in predicates {
            self.add_where_predicate(predicate);
        }
    }

    pub fn add_to_where_clause(&self, where_clause: Option<&WhereClause>) -> Option<WhereClause> {
        let predicates = self
            .0
            .iter()
            .filter(|(_, bounds)| !bounds.is_empty())
            .map(|(a, b)| (a.clone(), b.clone()))
            .map(|((lifetimes, bounded_ty), bounds)| {
                WherePredicate::Type(PredicateType {
                    lifetimes,
                    bounded_ty,
                    colon_token: Token![:](Span::mixed_site()),
                    bounds: bounds.into_iter().collect(),
                })
            })
            .peekable();

        // de-duplicate elements newly added and within existing where clause
        let predicates = predicates
            .chain(
                where_clause
                    .into_iter()
                    .flat_map(|where_clause| where_clause.predicates.clone()),
            )
            .chain(once(syn::parse_quote!(Self: ::std::error::Error)))
            .collect::<HashSet<_>>();

        Some(WhereClause {
            where_token: Token![where](Span::mixed_site()),
            predicates: predicates.into_iter().collect(),
        })
    }
}
