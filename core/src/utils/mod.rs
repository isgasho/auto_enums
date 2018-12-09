use proc_macro2::{Ident, Span};
use smallvec::SmallVec;
use syn::{Attribute, Block, Expr, ExprBlock, ExprCall, ExprPath, Path, PathSegment, Stmt};

#[macro_use]
mod error;

pub(crate) use self::error::*;

pub(crate) type Stack<T> = SmallVec<[T; 8]>;

pub(crate) fn default<T: Default>() -> T {
    T::default()
}

pub(crate) trait VecExt<T> {
    fn find_remove<P>(&mut self, predicate: P) -> Option<T>
    where
        P: FnMut(&T) -> bool;
}

impl<T> VecExt<T> for Vec<T> {
    fn find_remove<P>(&mut self, predicate: P) -> Option<T>
    where
        P: FnMut(&T) -> bool,
    {
        fn remove<T>(v: &mut Vec<T>, index: usize) -> T {
            match v.len() {
                1 => v.pop().unwrap(),
                2 => v.swap_remove(index),
                _ => v.remove(index),
            }
        }

        self.iter().position(predicate).map(|i| remove(self, i))
    }
}

pub(crate) fn ident_call_site<S: AsRef<str>>(s: S) -> Ident {
    Ident::new(s.as_ref(), Span::call_site())
}

pub(crate) fn path<I: Iterator<Item = PathSegment>>(segments: I) -> Path {
    Path {
        leading_colon: None,
        segments: segments.collect(),
    }
}

pub(crate) fn block(stmts: Vec<Stmt>) -> Block {
    Block {
        brace_token: default(),
        stmts,
    }
}

pub(crate) fn expr_block(block: Block) -> Expr {
    Expr::Block(ExprBlock {
        attrs: Vec::with_capacity(0),
        label: None,
        block,
    })
}

pub(crate) fn expr_call(attrs: Vec<Attribute>, path: Path, args: Expr) -> Expr {
    Expr::Call(ExprCall {
        attrs,
        func: Box::new(Expr::Path(ExprPath {
            attrs: Vec::with_capacity(0),
            qself: None,
            path,
        })),
        paren_token: default(),
        args: Some(args).into_iter().collect(),
    })
}