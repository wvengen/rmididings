extern crate proc_macro;
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::{quote};

#[proc_macro]
pub fn patch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    patch_ts(TokenStream::from(input), None).into()
}

fn patch_ts(input: TokenStream, connection: Option<&str>) -> TokenStream {
    let mut parts: Vec<TokenStream> = vec![];
    let mut cur_ts = TokenStream::new();
    let mut cur_connection = connection;
    let mut prev_token = None;
    let mut alter_next_ident_group = None;

    // dbg!(&input);

    for t in input {
        let next_prev_token = t.clone();
        match t {
            TokenTree::Group(ref g) => {
                if let Some(TokenTree::Ident(_)) = prev_token {
                    // A group after an identifier it's a function call, keep it here.
                    // Also handle unary operators.
                    match alter_next_ident_group {
                        Some(('*', Some(i))) => {
                            let mut ts = TokenStream::from(i);
                            ts.extend(TokenStream::from(t));
                            cur_ts.extend(make_connection(Some("Fork"), vec![quote!(Pass()), ts]));
                            alter_next_ident_group = None;
                        },
                        Some(('!', Some(i))) => {
                            cur_ts.extend(quote!(Not!(#i#t)));
                            alter_next_ident_group = None;
                        },
                        _ => {
                            cur_ts.extend(TokenStream::from(t));
                        }
                    }
                } else if g.delimiter() == Delimiter::Bracket {
                    // A group within square brackets is a Fork subexpression.
                    cur_ts.extend(patch_ts(g.stream(), Some("Fork")));
                } else if g.delimiter() == Delimiter::Parenthesis {
                    // A group within parentheses is a grouping expression.
                    cur_ts.extend(patch_ts(g.stream(), None));
                } else {
                    panic!("Unexpected group: {:?}", g);
                }
            },
            TokenTree::Punct(ref p) if p.as_char() == '/' => {
                // Fork with '/' 
                parts.push(cur_ts);
                cur_ts = TokenStream::new();

                if cur_connection != Some("Fork") {
                    // We have a new connection, replace current parts with a single item
                    parts = vec![make_connection(cur_connection, parts)];
                    cur_connection = Some("Fork");
                }
            },
            TokenTree::Punct(ref p) if p.as_char() == '>' => {
                parts.push(cur_ts);
                cur_ts = TokenStream::new();

                if cur_connection != Some("Chain") {
                    // We have a new connection, replace current parts with a single item
                    parts = vec![make_connection(cur_connection, parts)];
                    cur_connection = Some("Chain");
                }
            },
            TokenTree::Punct(ref p) if p.as_char() == ',' && connection.is_some() => {
                // Outer connection (fork with '[' and ']')
                // TODO handle multiple connections on a single level (!)
                parts.push(cur_ts);
                cur_ts = TokenStream::new();

                if cur_connection != connection {
                    // We have a new connection, replace current parts with a single item
                    parts = vec![make_connection(cur_connection, parts)];
                    cur_connection = Some("Fork");
                }
            },
            TokenTree::Punct(ref p) if ['*', '-', '!'].contains(&p.as_char()) => {
                // Prepare for handling unary operator as connection
                alter_next_ident_group = Some((p.as_char(), None));
                // Note that we don't add the token.
            },
            TokenTree::Ident(_) if alter_next_ident_group.is_some() => {
                // If we're altering the next ident-group combination, we need to output the ident in following group.
                let (p, _) = alter_next_ident_group.unwrap();
                alter_next_ident_group = Some((p, Some(t)));
            }
           _ => {
                cur_ts.extend(TokenStream::from(t));
            },
        }
        prev_token = Some(next_prev_token);
    }
    parts.push(cur_ts);
    let result = make_connection(cur_connection, parts);
    // dbg!(&result);

    result
}

fn make_connection(connection: Option<&str>, mut parts: Vec<TokenStream>) -> TokenStream {
    if parts.len() == 0 {
        return TokenStream::new();
    } else if parts.len() == 1 {
        return parts.pop().unwrap();
    } else if let Some(cc) = connection {
        if cc != "Unknown" {
            let cc_ident = Ident::new(cc, Span::call_site());
            return quote!(
                rmididings::proc::FilterChain::new(
                    rmididings::proc::ConnectionType::#cc_ident,
                    vec![#(::std::boxed::Box::new(#parts)),*]
                )
            );
        }
    }
    parts.into_iter().reduce(|mut a, b| { a.extend(b); a }).unwrap()
}
