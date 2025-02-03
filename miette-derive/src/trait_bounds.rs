use std::{
    collections::{HashMap, VecDeque},
    iter::{once, FromIterator},
};

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Plus, AngleBracketedGenericArguments, AssocType, BoundLifetimes,
    GenericArgument, GenericParam, Generics, ParenthesizedGenericArguments, Path, PathArguments,
    PathSegment, PredicateType, ReturnType, Token, TraitBound, Type, TypeArray, TypeGroup,
    TypeParamBound, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple, WhereClause,
    WherePredicate,
};

#[derive(Default)]
pub struct RequiredTraitBound {
    r#static: bool,
    std_error: bool,
    miette_diagnostic: bool,
    source_code: bool,
    into_source_span: bool,
    std_into_iter: bool,
    std_deref: bool,
    std_to_owned: bool,
}

impl RequiredTraitBound {
    fn to_bounds(&self) -> Punctuated<TypeParamBound, Plus> {
        let mut bounds = Punctuated::new();
        if self.std_error && !self.miette_diagnostic {
            bounds.push(TypeParamBound::Trait(syn::parse_quote!(
                ::std::error::Error
            )));
        }

        if self.miette_diagnostic {
            bounds.push(TypeParamBound::Trait(syn::parse_quote!(
                ::miette::Diagnostic
            )))
        }

        if self.source_code {
            bounds.push(TypeParamBound::Trait(syn::parse_quote!(
                ::miette::SourceCode
            )))
        }

        if self.into_source_span {
            bounds.push(TypeParamBound::Trait(syn::parse_quote!(
                ::std::convert::Into<::miette::SourceSpan>
            )))
        }

        if self.std_into_iter {
            bounds.push(TypeParamBound::Trait(syn::parse_quote!(
                ::std::iter::IntoIterator
            )))
        }

        if self.std_deref {
            bounds.push(TypeParamBound::Trait(syn::parse_quote!(::std::ops::Deref)))
        }

        if self.std_to_owned {
            bounds.push(TypeParamBound::Trait(syn::parse_quote!(
                ::std::borrow::ToOwned
            )))
        }

        if self.r#static {
            bounds.push(TypeParamBound::Lifetime(syn::parse_quote!('static)))
        }

        bounds
    }

    fn register_transparent_usage(&mut self) {
        self.r#static = true;
        self.miette_diagnostic = true;
    }

    fn register_source_code_usage(&mut self) {
        self.source_code = true;
    }

    fn register_label_usage(&mut self) {
        self.into_source_span = true;
    }

    fn register_collection_usage(&mut self) {
        self.std_into_iter = true;
    }
    fn register_related_item_usage(&mut self) {
        self.miette_diagnostic = true;
        self.r#static = true;
    }

    fn register_source_usage(&mut self) {
        self.miette_diagnostic = true;
        self.r#static = true;
    }

    fn register_deref_usage(&mut self) {
        self.std_deref = true;
    }

    fn register_to_owned_usage(&mut self) {
        self.std_to_owned = true;
    }
}

pub struct TraitBoundStore(HashMap<(Option<BoundLifetimes>, Type), RequiredTraitBound>);

impl TraitBoundStore {
    pub fn new(generics: &Generics) -> Self {
        let hash_map = generics
            .params
            .iter()
            .filter_map(|param| {
                if let GenericParam::Type(ty) = param {
                    Some(ty)
                } else {
                    None
                }
            })
            .map(|param| {
                Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(once(PathSegment {
                            ident: param.ident.clone(),
                            arguments: PathArguments::None,
                        })),
                    },
                })
            })
            .map(|v| ((None, v), RequiredTraitBound::default()))
            .collect::<HashMap<_, _>>();

        Self(hash_map)
    }

    pub fn extract_option(r#type: &Type) -> Option<&Type> {
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) = r#type
        {
            segments
                .last()
                .filter(|seg| seg.ident == "Option")
                .and_then(|seg| match &seg.arguments {
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => {
                        let mut iter = args.iter();

                        let ty = iter.next();
                        iter.next().xor(ty)
                    }
                    _ => None,
                })
                .and_then(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
        } else {
            None
        }
    }

    fn check_generic_usage<'ty>(&self, mut r#type: &'ty Type) -> Option<&'ty Type> {
        // in theory we could skip all this logic and just allow trivial bounds but that would add redundant trait bounds
        // to the derived impl - would be another choice to make. I choose to filter as much as possible so that we don't
        // introduce unneccessary bounds.

        // this reduces the type down as much as possible to remove unneeded groups.
        let original_type = loop {
            match r#type {
                Type::Paren(TypeParen { elem, .. }) => r#type = &**elem,
                Type::Group(TypeGroup { elem, .. }) => r#type = &**elem,
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
        to_check_queue.push_back((original_type, 0));

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

    pub fn merge_with(&self, where_clause: Option<&WhereClause>) -> Option<WhereClause> {
        let mut where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
            where_token: Token![where](Span::mixed_site()),
            predicates: Punctuated::new(),
        });

        where_clause
            .predicates
            .extend(self.0.iter().filter_map(|(ty, bounds)| {
                let bounds = bounds.to_bounds();
                (!bounds.is_empty()).then(|| {
                    WherePredicate::Type(PredicateType {
                        lifetimes: ty.0.clone(),
                        bounded_ty: ty.1.clone(),
                        colon_token: Token![:](Span::mixed_site()),
                        bounds,
                    })
                })
            }));

        where_clause
            .predicates
            .push(WherePredicate::Type(PredicateType {
                lifetimes: None,
                bounded_ty: Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(once(PathSegment {
                            ident: syn::Ident::new("Self", Span::mixed_site()),
                            arguments: PathArguments::None,
                        })),
                    },
                }),
                colon_token: syn::Token![:](Span::mixed_site()),
                bounds: Punctuated::from_iter(once(TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path: syn::parse_quote!(::std::error::Error),
                }))),
            }));

        Some(where_clause)
    }

    pub fn register_transparent_usage(&mut self, r#type: &Type) {
        let Some(r#type) = self.check_generic_usage(r#type) else {
            return;
        };

        let type_opts = self.0.entry((None, r#type.clone())).or_default();
        type_opts.register_transparent_usage()
    }

    pub fn register_source_code_usage(&mut self, r#type: &Type) {
        let Some(r#type) = self.check_generic_usage(r#type) else {
            return;
        };

        let type_opts = self.0.entry((None, r#type.clone())).or_default();
        type_opts.register_source_code_usage()
    }

    pub fn register_label_usage(&mut self, r#type: &Type) {
        let r#type = Self::extract_option(r#type).unwrap_or(r#type);

        let Some(ty) = self.check_generic_usage(r#type) else {
            return;
        };

        let type_opts = self.0.entry((None, ty.clone())).or_default();

        type_opts.register_to_owned_usage();

        let type_opts_to_owned = self
            .0
            .entry((
                None,
                syn::parse_quote!(<#ty as ::std::borrow::ToOwned>::Owned),
            ))
            .or_default();
        type_opts_to_owned.register_label_usage();
    }

    pub fn register_label_collection_usage(&mut self, r#type: &Type) {
        let Some(ty) = self.check_generic_usage(r#type) else {
            return;
        };

        let ty: syn::Type = syn::parse_quote!(&'__miette_internal_lt #ty);

        let type_opts = self
            .0
            .entry((
                Some(syn::parse_quote!(for<'__miette_internal_lt>)),
                ty.clone(),
            ))
            .or_default();
        type_opts.register_collection_usage();

        let type_opts_item = self
            .0
            .entry((
                Some(syn::parse_quote!(for<'__miette_internal_lt>)),
                syn::parse_quote!(<#ty as ::std::iter::IntoIterator>::Item),
            ))
            .or_default();
        type_opts_item.register_deref_usage();

        let type_opts_deref_item = self
            .0
            .entry((
                Some(syn::parse_quote!(for<'__miette_internal_lt>)),
                syn::parse_quote!(<<#ty as ::std::iter::IntoIterator>::Item as ::std::ops::Deref>::Target),
            ))
            .or_default();
        type_opts_deref_item.register_to_owned_usage();

        let type_opts_deref_to_owned_item = self
            .0
            .entry((
                Some(syn::parse_quote!(for<'__miette_internal_lt>)),
                syn::parse_quote!(<<<#ty as ::std::iter::IntoIterator>::Item as ::std::ops::Deref>::Target as ::std::borrow::ToOwned>::Owned),
            ))
            .or_default();
        type_opts_deref_to_owned_item.register_label_usage();
    }

    pub fn register_related_usage(&mut self, r#type: &Type) {
        let Some(ty) = self.check_generic_usage(r#type) else {
            return;
        };

        // this is somewhat hacky and only supports concrete types for the #[related] type
        // ittself but supports generics for the arguments, i.e. Vec<T> where T is generic.
        //
        // I think that this is a current limitation of the design of the Diagnostic trait,
        // since we'd need bounds on the method and we can't do that (to refer to the lifetime)
        //
        // Someone smarter than me might be able to figure out a better solution (?)
        let type_opts_item = self
            .0
            .entry((
                None,
                syn::parse_quote!(<#ty as ::std::iter::IntoIterator>::Item),
            ))
            .or_default();
        type_opts_item.register_related_item_usage();
    }

    pub fn register_source_usage(&mut self, r#type: &Type) {
        let Some(ty) = self.check_generic_usage(r#type) else {
            return;
        };

        let type_opts = self.0.entry((None, ty.clone())).or_default();
        type_opts.register_source_usage();
    }
}
