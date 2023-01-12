use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn command(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let fn_visiblity = &input.vis;
    let fn_name = &input.sig.ident;
    let fn_args = &input.sig.inputs;
    let fn_ret = &input.sig.output;
    let fn_body = &input.block;

    let fn_args = fn_args.iter().clone();

    let mut arguments: Vec<TokenStream2> = Vec::new();
    let mut client_ident = None;
    let mut world_ident = None;

    // validate the types of all arguments, the can only be i64, f64, String or bool
    for arg in fn_args {
        match arg {
            syn::FnArg::Typed(t) => {
                let ty = &t.ty;
                match *ty.clone() {
                    syn::Type::Path(p) => {
                        let path = &p.path;

                        if path.segments.len() != 1 {
                            panic!("Invalid argument type, segments must have a length of 1");
                        }

                        let segment = &path.segments[0];

                        let mut is_optional = false;

                        let var_name = match *t.pat.clone() {
                            syn::Pat::Ident(i) => i.ident.to_string(),
                            _ => panic!("Invalid argument name"),
                        };

                        // Client<Game> is allowed
                        if segment.ident == "Client" {
                            client_ident = Some(format_ident!("{}", var_name));
                            continue;
                        }

                        if segment.ident == "World" {
                            world_ident = Some(format_ident!("{}", var_name));
                            continue;
                        }

                        // check if it's an Option
                        if segment.ident == "Option" {
                            is_optional = true;
                            match segment.arguments {
                                syn::PathArguments::AngleBracketed(ref a) => {
                                    if a.args.len() != 1 {
                                        panic!(
                                            "Invalid argument type, Option must have a length of 1"
                                        );
                                    }

                                    let arg = &a.args[0];

                                    match arg {
                                        syn::GenericArgument::Type(ref t) => match t.clone() {
                                            syn::Type::Path(p) => {
                                                let path = &p.path;

                                                if path.segments.len() != 1 {
                                                    panic!("Invalid argument type, segments must have a length of 1");
                                                }

                                                let segment = &path.segments[0];

                                                if segment.ident != "i64"
                                                    && segment.ident != "f64"
                                                    && segment.ident != "String"
                                                    && segment.ident != "bool"
                                                {
                                                    panic!(
                                                        "Invalid argument type {}",
                                                        segment.ident
                                                    );
                                                }
                                            }
                                            _ => panic!("Invalid argument type, must be a path"),
                                        },
                                        _ => panic!("Invalid argument type, must be a type"),
                                    }
                                }
                                _ => panic!("Invalid argument type, must be an angle bracketed"),
                            }
                        } else if segment.ident != "i64"
                            && segment.ident != "f64"
                            && segment.ident != "String"
                            && segment.ident != "bool"
                            && segment.ident != "Client"
                        {
                            panic!("Invalid argument type {}", segment.ident);
                        }

                        let arg_index = arguments.len() + 1;

                        let mut a = quote::quote! {};

                        // extract the type name as string from p
                        let type_name = match segment.ident.to_string().as_str() {
                            "i64" => "Integer",
                            "f64" => "Float",
                            "String" => "String",
                            "bool" => "Boolean",
                            _ => "Unknown Type",
                        };

                        if is_optional {
                            a.extend(quote::quote! {
                                let #t = match __args.pop_front() {
                                    Some(v) => match v.try_into() {
                                        Ok(v) => Some(v),
                                        Err(e) => return Err(format!("Invalid argument #{}: {}", #arg_index, e).into()),
                                    },
                                    None => None,
                                };
                            });
                        } else {
                            a.extend(quote::quote! {
                                // args is a vec
                                let #t = match __args.pop_front() {
                                    Some(v) => match v.try_into() {
                                        Ok(v) => v,
                                        Err(e) => return Err(format!("Invalid argument #{}: {}", #arg_index, e).into()),
                                    },
                                    None => return Err(format!("Missing argument #{}: {}({})", #arg_index, #type_name, #var_name).into()),
                                };
                            });
                        }

                        arguments.push(a);
                    }
                    _ => panic!("Invalid argument type, must be a path"),
                }
            }
            _ => panic!("'self' fns not supported."),
        }
    }

    let client_ident = client_ident.unwrap_or(format_ident!("__client"));
    let world_ident = world_ident.unwrap_or(format_ident!("__world"));

    quote::quote! {
        #fn_visiblity fn #fn_name(mut __args: ::piquant_command::Arguments, #client_ident: &mut Client<Game>, #world_ident: &mut World<Game>) #fn_ret {
            #(#arguments)*

            #fn_body
        }
    }
    .into()
}
