use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::GenericArgument;
use syn::parse::{Parse};

#[proc_macro]
pub fn type_pipe(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as PipeInput);
    pipe(input).into()
}

#[proc_macro]
pub fn type_pipe_pre(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as PipeInput);
    pipe_pre(input).into()
}

#[proc_macro]
pub fn type_pipe_post(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as PipeInput);
    pipe_post(input).into()
}

fn pipe(input: PipeInput) -> TokenStream {
    input.types
        .into_iter()
        .reduce(
            |ty, mut item| {
                replace_recur(&mut item, |i| {
                    if matches!(i, syn::Type::Infer(_)) {
                        *i = ty.clone();
                    }
                });

                item
            },
        )
        .map(ToTokens::into_token_stream)
        .unwrap_or_default()
}

fn pipe_pre(input: PipeInput) -> TokenStream {
    pipe_impl(input, false)
}

fn pipe_post(input: PipeInput) -> TokenStream {
    pipe_impl(input, true)
}

fn pipe_impl(input: PipeInput, push: bool) -> TokenStream {
    input.types
        .into_iter()
        .reduce(
            |ty, mut item| {
                let syn::Type::Path(path) = &mut item else {
                    return item;
                };

                let Some(segment) = path.path.segments.last_mut() else {
                    return item;
                };

                match &mut segment.arguments {
                    syn::PathArguments::None => {
                        segment.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote! {
                            <#ty>
                        });
                    }
                    syn::PathArguments::AngleBracketed(args) => {
                        if push {
                            args.args.push(GenericArgument::Type(ty.clone()));
                        } else {
                            args.args.insert(0, GenericArgument::Type(ty.clone()));
                        }
                    }
                    syn::PathArguments::Parenthesized(_) => {
                        panic!("Parenthesised functions cannot be piped. [Can't apply types to signatures like: `Fn()`]")
                    }
                }

                item
            },
        )
        .map(ToTokens::into_token_stream)
        .unwrap_or_default()
}

struct PipeInput {
    types: syn::punctuated::Punctuated<syn::Type, syn::Token![,]>,
}

impl Parse for PipeInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            types: input.parse_terminated(syn::Type::parse, syn::Token![,])?,
        })
    }
}

fn replace_recur<F>(item: &mut syn::Type, func: F)
where
    F: for<'a> Fn(&'a mut syn::Type),
{
    struct Replacer<F> {
        func: F,
    }

    impl<F> syn::visit_mut::VisitMut for Replacer<F>
    where
        F: for<'a> Fn(&'a mut syn::Type),
    {
        fn visit_type_mut(&mut self, i: &mut syn::Type) {
            syn::visit_mut::visit_type_mut(self, i);

            (self.func)(i)
        }
    }

    let mut visitor = Replacer {
        func,
    };

    syn::visit_mut::VisitMut::visit_type_mut(&mut visitor, item);
}

mod tests {
    use super::*;

    macro_rules! check {
        (
            $func:path,
            $input:tt,
            $expected:tt
            $(,)?
        ) => {{
            let input = {
                quote::quote! $input
            };
            let expected = {
                quote::quote! $expected
            };

            let input: PipeInput = syn::parse2(input).expect("invalid syntax");

            let ts = $func(input);

            assert_eq!(ts.to_string(), expected.to_string());
        }};
    }

    #[test]
    fn simple_pipe() {
        check! {
            pipe,
            {
                T,
                MyType<_>
            },
            {
                MyType<T>
            },
        }
        check! {
            pipe,
            {
                T,
                Wrapper<_>,
                MyType<_, _>
            },
            {
                MyType<Wrapper<T>, Wrapper<T> >
            },
        }
    }

    #[test]
    fn simple_pipe_pre() {
        check! {
            pipe_pre,
            {
                T,
                MyType
            },
            {
                MyType<T>
            },
        }
        check! {
            pipe_pre,
            {
                T,
                MyType<String>
            },
            {
                MyType<T, String>
            },
        }
    }

    #[test]
    fn simple_pipe_post() {
        check! {
            pipe_post,
            {
                T,
                MyType
            },
            {
                MyType<T>
            },
        }
        check! {
            pipe_post,
            {
                T,
                MyType<String>
            },
            {
                MyType<String, T>
            },
        }
        check! {
            pipe_post,
            {
                T,
                MyType<Wrapped<String>>
            },
            {
                MyType<Wrapped<String>, T>
            },
        }
    }

    #[test]
    fn large_pipe() {
        check! {
            pipe,
            {
                T,
                MyType<_>,
                Layer<_, String, _>,
                AnotherType<_, _, _>,
            },
            {
                AnotherType<
                    Layer<
                        MyType<T>,
                        String,
                        MyType<T>
                    >,
                    Layer<
                        MyType<T>,
                        String,
                        MyType<T>
                    >,
                    Layer<
                        MyType<T>,
                        String,
                        MyType<T>
                    >
                >
            },
        }
    }
}

