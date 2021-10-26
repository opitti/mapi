use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed, ItemFn,Expr,parse_quote,Stmt};
use syn::{Ident, Local, Pat, Token};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use std::collections::HashSet;
use syn::fold::{self, Fold};

// using proc_macro_attribute to declare an attribute like procedural macro
#[proc_macro_attribute]
// _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
pub fn my_custom_attribute(_metadata: TokenStream, _input: TokenStream) -> TokenStream {
    // returing a simple TokenStream for Struct
    TokenStream::from(quote!{struct H{}})
}

struct Args{
    vars:HashSet<Ident>
}

impl Args {
    fn should_print_expr(&self, e: &Expr) -> bool {
        match *e {
            Expr::Path(ref e) => {
                // variable shouldn't start wiht ::
                if e.path.leading_colon.is_some() {
                    false
                // should be a single variable like `x=8` not n::x=0
                } else if e.path.segments.len() != 1 {
                    false
                } else {
                    // get the first part
                    let first = e.path.segments.first().unwrap();
                    // check if the variable name is in the Args.vars hashset
                    self.vars.contains(&first.ident) && first.arguments.is_empty()
                }
            }
            _ => false,
        }
    }

// used for checking if to print let i=0 etc or not
    fn should_print_pat(&self, p: &Pat) -> bool {
        match p {
// check if variable name is present in set
            Pat::Ident(ref p) => self.vars.contains(&p.ident),
            _ => false,
        }
    }

// manipulate tree to insert print statement
    fn assign_and_print(&mut self, left: Expr, op: &dyn ToTokens, right: Expr) -> Expr {
 // recurive call on right of the assigment statement
        let right = fold::fold_expr(self, right);
// returning manipulated sub-tree
        parse_quote!({
            #left #op #right;
            println!(concat!(stringify!(#left), " = {:?}"), #left);
        })
    }

// manipulating let statement
    fn let_and_print(&mut self, local: Local) -> Stmt {
        let Local { pat, init, .. } = local;
        let init = self.fold_expr(*init.unwrap().1);
// get the variable name of assigned variable
        let ident = match pat {
            Pat::Ident(ref p) => &p.ident,
            _ => unreachable!(),
        };
// new sub tree
        parse_quote! {
            let #pat = {
                #[allow(unused_mut)]
                let #pat = #init;
                println!(concat!(stringify!(#ident), " = {:?}"), #ident);
                #ident
            };
        }
    }
}


impl Fold for Args {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
// for changing assignment like a=5
            Expr::Assign(e) => {
// check should print
                if self.should_print_expr(&e.left) {
                    self.assign_and_print(*e.left, &e.eq_token, *e.right)
                } else {
// continue with default travesal using default methods
                    Expr::Assign(fold::fold_expr_assign(self, e))
                }
            }
// for changing assigment and operation like a+=1
            Expr::AssignOp(e) => {
// check should print
                if self.should_print_expr(&e.left) {
                    self.assign_and_print(*e.left, &e.op, *e.right)
                } else {
// continue with default behaviour
                    Expr::AssignOp(fold::fold_expr_assign_op(self, e))
                }
            }
// continue with default behaviour for rest of expressions
            _ => fold::fold_expr(self, e),
        }
    }

// for let statements like let d=9
    fn fold_stmt(&mut self, s: Stmt) -> Stmt {
        match s {
            Stmt::Local(s) => {
                if s.init.is_some() && self.should_print_pat(&s.pat) {
                    self.let_and_print(s)
                } else {
                    Stmt::Local(fold::fold_local(self, s))
                }
            }
            _ => fold::fold_stmt(self, s),
        }
    }
}


impl Parse for Args{
    fn parse(input: ParseStream) -> Result<Self> {
        // parses a,b,c, or a,b,c where a,b and c are Indent
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}



#[proc_macro_attribute]
pub fn trace_var(args: TokenStream, input: TokenStream) -> TokenStream {
// parse the input
    let input = parse_macro_input!(input as ItemFn);
// parse the arguments
    let mut args = parse_macro_input!(args as Args);
// create the ouput
    let output = args.fold_item_fn(input);
// return the TokenStream
    TokenStream::from(quote!(#output))
}




#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let description = match data {
    syn::Data::Struct(s) => match s.fields {
        syn::Fields::Named(FieldsNamed { named, .. }) => {
        let idents = named.iter().map(|f| &f.ident);

        format!(
            "a struct with these named fields: {}",
            quote! {#(#idents), *}
        )
        }

        syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
        let num_fields = unnamed.iter().count();
        format!("a struct with {} unnamed fields", num_fields)
        }
        syn::Fields::Unit => format!("a unit struct"),
    },
    syn::Data::Enum(DataEnum { variants, .. }) => {
        let vs = variants.iter().map(|v| &v.ident);
        format!("an enum with these variants: {}", quote! {#(#vs),*})
    }
    syn::Data::Union(DataUnion {
        fields: FieldsNamed { named, .. },
        ..
    }) => {
        let idents = named.iter().map(|f| &f.ident);
        format!("a union with these named fields: {}", quote! {#(#idents),*})
    }
    };

    let output = quote! {
    impl #ident {
        fn describe() {
        println!("{} is {}.", stringify!(#ident), #description);
        }
    }
    };

    output.into()
}