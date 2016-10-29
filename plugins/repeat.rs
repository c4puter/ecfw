#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use syntax::codemap::Span;
use syntax::tokenstream::TokenTree;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use rustc_plugin::Registry;
use syntax::util::small_vector::SmallVector;
use syntax::ast;
use syntax::ast::Lit;
use std::error::Error;

fn expand_repeat(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(tts);
    let times = match parser.parse_lit() {
        Ok(lit) => match lit.node {
            ast::LitKind::Int(n, _) => n,
            _ => {
                cx.span_err(lit.span, "Expected literal integer");
                return DummyResult::any(sp);
            }
        },
        Err(e) => {
            cx.span_err(sp, &format!("{:?}", e));
            return DummyResult::any(sp);
        }
    };
    let res = parser.parse_block();

    match res {
        Ok(block) => {
            let mut stmts = SmallVector::many(block.stmts.clone());
            for _ in 1..times {
                let rep_stmts = SmallVector::many(block.stmts.clone());
                stmts.push_all(rep_stmts);
            }
            MacEager::stmts(stmts)
        }
        Err(e) => {
            cx.span_err(sp, &format!("{:?}", e));
            DummyResult::any(sp)
        }
    }
}

fn expand_repeat_expr(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(tts);
    let times = match parser.parse_lit() {
        Ok(lit) => match lit.node {
            ast::LitKind::Int(n, _) => n,
            _ => {
                cx.span_err(lit.span, "Expected literal integer");
                return DummyResult::any(sp);
            }
        },
        Err(e) => {
            cx.span_err(sp, &format!("{:?}", e));
            return DummyResult::any(sp);
        }
    };

    match parser.parse_expr() {
        Ok(expr) => {
            let mut exprs = Vec::<P<ast::Expr>>::new();
            for _ in 0..times {
                exprs.push(expr.clone());
            }
            let expr_vec = cx.expr_vec(sp, exprs);
            MacEager::expr(expr_vec)
        }
        Err(e) => {
            cx.span_err(sp, &format!("{:?}", e));
            DummyResult::any(sp)
        }
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("repeat", expand_repeat);
    reg.register_macro("repeat_expr", expand_repeat_expr);
}
