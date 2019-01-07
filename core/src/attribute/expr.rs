use std::cell::Cell;

use syn::{
    visit_mut::{self, VisitMut},
    *,
};

use crate::utils::{Result, *};

use super::*;

const REC_ATTR: &str = "rec";

pub(super) const NEVER_ATTR: &str = "never";

pub(super) const EMPTY_ATTRS: &[&str] = &[NEVER_ATTR, REC_ATTR];

#[derive(Debug)]
struct Params<'a> {
    marker: &'a Marker,
    rec: Cell<bool>,
}

impl<'a> From<&'a super::Params> for Params<'a> {
    fn from(params: &'a super::Params) -> Self {
        Self {
            marker: params.marker(),
            rec: Cell::new(false),
        }
    }
}

fn last_expr<T, F, OP>(expr: &Expr, success: T, mut filter: F, op: OP) -> T
where
    F: FnMut(&Expr) -> bool,
    OP: FnOnce(&Expr) -> T,
{
    if !filter(expr) {
        return success;
    }

    match expr {
        Expr::Block(ExprBlock { block, .. }) | Expr::Unsafe(ExprUnsafe { block, .. }) => {
            match block.stmts.last() {
                Some(Stmt::Expr(expr)) => return last_expr(expr, success, filter, op),
                Some(Stmt::Semi(expr, _)) => {
                    if !filter(expr) {
                        return success;
                    }
                }
                Some(_) => return success,
                None => {}
            }
        }
        _ => {}
    }

    op(expr)
}

fn last_expr_mut<T, U, F, OP>(expr: &mut Expr, state: U, success: T, mut filter: F, op: OP) -> T
where
    F: FnMut(&Expr, &U) -> bool,
    OP: FnOnce(&mut Expr, U) -> T,
{
    if !filter(expr, &state) {
        return success;
    }

    match expr {
        Expr::Block(ExprBlock { block, .. }) | Expr::Unsafe(ExprUnsafe { block, .. }) => {
            match block.stmts.last_mut() {
                Some(Stmt::Expr(expr)) => return last_expr_mut(expr, state, success, filter, op),
                Some(Stmt::Semi(expr, _)) => {
                    if !filter(expr, &state) {
                        return success;
                    }
                }
                Some(_) => return success,
                None => {}
            }
        }
        _ => {}
    }

    op(expr, state)
}

fn is_unreachable(expr: &Expr, builder: &Builder, params: &Params) -> bool {
    const UNREACHABLE_MACROS: &[&str] = &["unreachable", "panic"];

    last_expr(
        expr,
        true,
        |expr| !expr.any_empty_attr(NEVER_ATTR) && !expr.any_attr(NAME),
        |expr| match expr {
            Expr::Break(_) | Expr::Continue(_) | Expr::Return(_) => true,

            Expr::Macro(ExprMacro { mac, .. }) => {
                UNREACHABLE_MACROS.iter().any(|i| mac.path.is_ident(i))
                    || params.marker.marker_macro(mac)
            }

            Expr::Call(ExprCall { args, func, .. }) if args.len() == 1 => match &**func {
                Expr::Path(path) => {
                    path.qself.is_none()
                        && path.path.leading_colon.is_none()
                        && path.path.segments.len() == 2
                        && path.path.segments[0].arguments.is_empty()
                        && path.path.segments[1].arguments.is_empty()
                        && path.path.segments[0].ident == builder.ident()
                }
                _ => false,
            },

            Expr::Match(ExprMatch { arms, .. }) => arms.iter().all(|arm| {
                arm.any_empty_attr(NEVER_ATTR) || is_unreachable(&*arm.body, builder, params)
            }),

            Expr::Try(ExprTry { expr, .. }) => match &**expr {
                Expr::Path(path) => path.qself.is_none() && path.path.is_ident("None"),
                Expr::Call(ExprCall { args, func, .. }) if args.len() == 1 => match &**func {
                    Expr::Path(path) => path.qself.is_none() && path.path.is_ident("Err"),
                    _ => false,
                },
                _ => false,
            },

            _ => false,
        },
    )
}

pub(super) fn child_expr(
    expr: &mut Expr,
    builder: &mut Builder,
    params: &super::Params,
) -> Result<()> {
    fn _child_expr(expr: &mut Expr, builder: &mut Builder, params: &Params) -> Result<()> {
        last_expr_mut(
            expr,
            builder,
            Ok(()),
            |expr, builder| {
                if expr.any_empty_attr(REC_ATTR) {
                    params.rec.set(true);
                }
                !is_unreachable(expr, builder, params)
            },
            |expr, builder| match expr {
                Expr::Match(expr) => expr_match(expr, builder, params),
                Expr::If(expr) => expr_if(expr, builder, params),
                Expr::Loop(expr) => expr_loop(expr, builder, params),
                Expr::MethodCall(expr) => _child_expr(&mut *expr.receiver, builder, params),
                _ => Ok(()),
            },
        )
    }

    _child_expr(expr, builder, &Params::from(params))
}

fn rec_attr(expr: &mut Expr, builder: &mut Builder, params: &Params) -> Result<bool> {
    last_expr_mut(
        expr,
        builder,
        Ok(true),
        |expr, builder| !is_unreachable(expr, builder, params),
        |expr, builder| match expr {
            Expr::Match(expr) => expr_match(expr, builder, params).map(|_| true),
            Expr::If(expr) => expr_if(expr, builder, params).map(|_| true),
            Expr::Loop(expr) => expr_loop(expr, builder, params).map(|_| true),
            _ => Ok(false),
        },
    )
}

fn expr_match(expr: &mut ExprMatch, builder: &mut Builder, params: &Params) -> Result<()> {
    fn skip(arm: &mut Arm, builder: &mut Builder, params: &Params) -> Result<bool> {
        Ok(arm.any_empty_attr(NEVER_ATTR)
            || is_unreachable(&*arm.body, &builder, params)
            || ((arm.any_empty_attr(REC_ATTR) || params.rec.get())
                && rec_attr(&mut *arm.body, builder, params)?))
    }

    expr.arms.iter_mut().try_for_each(|arm| {
        if !skip(arm, builder, params)? {
            arm.comma = Some(default());
            replace_expr(&mut *arm.body, |x| builder.next_expr(x));
        }

        Ok(())
    })
}

fn expr_if(expr: &mut ExprIf, builder: &mut Builder, params: &Params) -> Result<()> {
    fn skip(last: Option<&mut Stmt>, builder: &mut Builder, params: &Params) -> Result<bool> {
        Ok(match &last {
            Some(Stmt::Expr(expr)) | Some(Stmt::Semi(expr, _)) => {
                is_unreachable(expr, &builder, params)
            }
            _ => true,
        } || match last {
            Some(Stmt::Expr(expr)) => {
                (expr.any_empty_attr(REC_ATTR) || params.rec.get())
                    && rec_attr(expr, builder, params)?
            }
            _ => true,
        })
    }

    fn replace_branch(branch: &mut Block, builder: &mut Builder) {
        replace_block(branch, |branch| {
            block(vec![Stmt::Expr(builder.next_expr(expr_block(branch)))])
        });
    }

    if !skip(expr.then_branch.stmts.last_mut(), builder, params)? {
        replace_branch(&mut expr.then_branch, builder);
    }

    match expr.else_branch.as_mut().map(|(_, expr)| &mut **expr) {
        Some(Expr::Block(expr)) => {
            if !skip(expr.block.stmts.last_mut(), builder, params)? {
                replace_branch(&mut expr.block, builder);
            }

            Ok(())
        }
        Some(Expr::If(expr)) => expr_if(expr, builder, params),
        Some(_) => Err(invalid_expr("after of `else` required `{` or `if`"))?,
        None => Err(invalid_expr("`if` expression missing an else clause"))?,
    }
}

fn expr_loop(expr: &mut ExprLoop, builder: &mut Builder, params: &Params) -> Result<()> {
    LoopVisitor::new(params.marker, &expr, builder).visit_block_mut(&mut expr.body);

    Ok(())
}

struct LoopVisitor<'a> {
    marker: &'a Marker,
    builder: &'a mut Builder,
    depth: usize,
    label: Option<Lifetime>,
}

impl<'a> LoopVisitor<'a> {
    fn new(marker: &'a Marker, expr: &ExprLoop, builder: &'a mut Builder) -> Self {
        Self {
            marker,
            builder,
            depth: 0,
            label: expr.label.as_ref().map(|l| l.name.clone()),
        }
    }

    fn label_eq(&self, other: Option<&Lifetime>) -> bool {
        match (&self.label, other) {
            (None, None) => true,
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    fn loop_bounds<E, F: FnOnce(&mut Self, E)>(&mut self, expr: E, f: F) {
        if self.label.is_some() {
            self.depth += 1;
            f(self, expr);
            self.depth -= 1;
        }
    }
}

impl<'a> VisitMut for LoopVisitor<'a> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if !expr.any_empty_attr(NEVER_ATTR) {
            match expr {
                // Stop at closure bounds
                Expr::Closure(_) => {}

                // Other loop bounds
                Expr::Loop(expr) => self.loop_bounds(expr, visit_mut::visit_expr_loop_mut),
                Expr::ForLoop(expr) => self.loop_bounds(expr, visit_mut::visit_expr_for_loop_mut),
                Expr::While(expr) => self.loop_bounds(expr, visit_mut::visit_expr_while_mut),

                // `break` in loop
                Expr::Break(br) => {
                    if (self.depth == 0 && br.label.is_none()) || self.label_eq(br.label.as_ref()) {
                        let expr = match br.expr.take().map_or_else(unit, |e| *e) {
                            Expr::Macro(expr) => {
                                if self.marker.marker_macro(&expr.mac) {
                                    Expr::Macro(expr)
                                } else {
                                    self.builder.next_expr(Expr::Macro(expr))
                                }
                            }
                            expr => self.builder.next_expr(expr),
                        };
                        br.expr = Some(Box::new(expr));
                    }

                    visit_mut::visit_expr_break_mut(self, br);
                }

                expr => visit_mut::visit_expr_mut(self, expr),
            }
        }
    }

    // Stop at item bounds
    fn visit_item_mut(&mut self, _item: &mut Item) {}
}
