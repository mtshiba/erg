//! implements `ASTLowerer`.
//!
//! ASTLowerer(ASTからHIRへの変換器)を実装
use erg_common::color::{GREEN, RED, RESET};
use erg_common::error::Location;
use erg_common::get_hash;
use erg_common::traits::{HasType, Locational, Stream};
use erg_common::ty::{ParamTy, Type};
use erg_common::{fn_name, log, switch_lang};

use erg_parser::ast;
use erg_parser::ast::AST;

use crate::context::{Context, ContextKind, RegistrationMode};
use crate::error::{LowerError, LowerErrors, LowerResult, LowerWarnings};
use crate::hir;
use crate::hir::HIR;
use crate::varinfo::Visibility;
use Visibility::*;

/// Singleton that checks types of an AST, and convert (lower) it into a HIR
#[derive(Debug)]
pub struct ASTLowerer {
    pub(crate) ctx: Context,
    errs: LowerErrors,
    warns: LowerWarnings,
}

impl ASTLowerer {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(
                "<module>".into(),
                ContextKind::Module,
                vec![],
                Some(Context::init_builtins()),
                vec![],
                vec![],
                Context::TOP_LEVEL,
            ),
            errs: LowerErrors::empty(),
            warns: LowerWarnings::empty(),
        }
    }

    fn return_t_check(
        &self,
        loc: Location,
        name: &str,
        expect: &Type,
        found: &Type,
    ) -> LowerResult<()> {
        self.ctx.unify(expect, found, Some(loc), None).map_err(|_| {
            LowerError::type_mismatch_error(
                line!() as usize,
                loc,
                self.ctx.caused_by(),
                name,
                expect,
                found,
            )
        })
    }

    fn use_check(&self, expr: hir::Expr, mode: &str) -> LowerResult<hir::Expr> {
        if mode != "eval" && !expr.ref_t().is_nonelike() {
            Err(LowerError::syntax_error(
                0,
                expr.loc(),
                self.ctx.name.clone(),
                switch_lang!(
                    "japanese" => "式の評価結果が使われていません",
                    "simplified_chinese" => "表达式评估结果未使用",
                    "traditional_chinese" => "表達式評估結果未使用",
                    "english" => "the evaluation result of the expression is not used",
                ),
                Some(
                    switch_lang!(
                        "japanese" => "値を使わない場合は、discard関数を使用してください",
                        "simplified_chinese" => "如果您不想使用该值，请使用discard函数",
                        "traditional_chinese" => "如果您不想使用該值，請使用discard函數",
                        "english" => "if you don't use the value, use discard function",
                    )
                    .into(),
                ),
            ))
        } else {
            Ok(expr)
        }
    }

    fn pop_append_errs(&mut self) {
        if let Err(mut errs) = self.ctx.pop() {
            self.errs.append(&mut errs);
        }
    }

    fn lower_array(&mut self, array: ast::Array, check: bool) -> LowerResult<hir::Array> {
        log!("[DEBUG] entered {}({array})", fn_name!());
        let mut hir_array = hir::Array::new(
            array.l_sqbr,
            array.r_sqbr,
            self.ctx.level,
            hir::Args::empty(),
            None,
        );
        for elem in array.elems.into_iters().0 {
            hir_array.push(self.lower_expr(elem.expr, check)?);
        }
        Ok(hir_array)
    }

    /// call全体で推論できる場合があり、そのときはcheck: falseにする
    fn lower_acc(&mut self, acc: ast::Accessor, check: bool) -> LowerResult<hir::Accessor> {
        log!("[DEBUG] entered {}({acc})", fn_name!());
        match acc {
            ast::Accessor::Local(n) => {
                let (t, __name__) = if check {
                    (
                        self.ctx.get_var_t(&n.symbol, &self.ctx.name)?,
                        self.ctx.get_local_uniq_obj_name(&n.symbol),
                    )
                } else {
                    (Type::ASTOmitted, None)
                };
                let acc = hir::Accessor::Local(hir::Local::new(n.symbol, __name__, t));
                Ok(acc)
            }
            ast::Accessor::Attr(a) => {
                let obj = self.lower_expr(*a.obj, true)?;
                let t = if check {
                    self.ctx.get_attr_t(&obj, &a.name.symbol, &self.ctx.name)?
                } else {
                    Type::ASTOmitted
                };
                let acc = hir::Accessor::Attr(hir::Attribute::new(obj, a.name.symbol, t));
                Ok(acc)
            }
            _ => todo!(),
        }
    }

    fn lower_bin(&mut self, bin: ast::BinOp) -> LowerResult<hir::BinOp> {
        log!("[DEBUG] entered {}({bin})", fn_name!());
        let mut args = bin.args.into_iter();
        let lhs = hir::PosArg::new(self.lower_expr(*args.next().unwrap(), true)?);
        let rhs = hir::PosArg::new(self.lower_expr(*args.next().unwrap(), true)?);
        let args = [lhs, rhs];
        let t = self.ctx.get_binop_t(&bin.op, &args, &self.ctx.name)?;
        let mut args = args.into_iter();
        let lhs = args.next().unwrap().expr;
        let rhs = args.next().unwrap().expr;
        Ok(hir::BinOp::new(bin.op, lhs, rhs, t))
    }

    fn lower_unary(&mut self, unary: ast::UnaryOp) -> LowerResult<hir::UnaryOp> {
        log!("[DEBUG] entered {}({unary})", fn_name!());
        let mut args = unary.args.into_iter();
        let arg = hir::PosArg::new(self.lower_expr(*args.next().unwrap(), true)?);
        let args = [arg];
        let t = self.ctx.get_unaryop_t(&unary.op, &args, &self.ctx.name)?;
        let mut args = args.into_iter();
        let expr = args.next().unwrap().expr;
        Ok(hir::UnaryOp::new(unary.op, expr, t))
    }

    fn lower_call(&mut self, call: ast::Call) -> LowerResult<hir::Call> {
        log!("[DEBUG] entered {}({}(...))", fn_name!(), call.obj);
        let (pos_args, kw_args, paren) = call.args.deconstruct();
        let mut hir_args = hir::Args::new(
            Vec::with_capacity(pos_args.len()),
            Vec::with_capacity(kw_args.len()),
            paren,
        );
        for arg in pos_args.into_iter() {
            hir_args.push_pos(hir::PosArg::new(self.lower_expr(arg.expr, true)?));
        }
        for arg in kw_args.into_iter() {
            hir_args.push_kw(hir::KwArg::new(
                arg.keyword,
                self.lower_expr(arg.expr, true)?,
            ));
        }
        let obj = self.lower_expr(*call.obj, false)?;
        let t = self
            .ctx
            .get_call_t(&obj, &hir_args.pos_args, &hir_args.kw_args, &self.ctx.name)?;
        Ok(hir::Call::new(obj, hir_args, t))
    }

    fn lower_lambda(&mut self, lambda: ast::Lambda) -> LowerResult<hir::Lambda> {
        log!("[DEBUG] entered {}({lambda})", fn_name!());
        let is_procedural = lambda.is_procedural();
        let id = get_hash(&lambda.sig);
        let name = format!("<lambda_{id}>");
        let kind = if is_procedural {
            ContextKind::Proc
        } else {
            ContextKind::Func
        };
        self.ctx.grow(&name, kind, Private)?;
        self.ctx
            .assign_params(&lambda.sig.params, None)
            .map_err(|e| {
                self.pop_append_errs();
                e
            })?;
        self.ctx
            .preregister(lambda.body.ref_payload())
            .map_err(|e| {
                self.pop_append_errs();
                e
            })?;
        let body = self.lower_block(lambda.body).map_err(|e| {
            self.pop_append_errs();
            e
        })?;
        let (non_default_params, default_params): (Vec<_>, Vec<_>) = self
            .ctx
            .params
            .iter()
            .partition(|(_, v)| v.kind.has_default());
        let non_default_params = non_default_params
            .into_iter()
            .map(|(name, vi)| {
                ParamTy::new(name.as_ref().map(|n| n.inspect().clone()), vi.t.clone())
            })
            .collect();
        let default_params = default_params
            .into_iter()
            .map(|(name, vi)| {
                ParamTy::new(name.as_ref().map(|n| n.inspect().clone()), vi.t.clone())
            })
            .collect();
        let bounds = self
            .ctx
            .instantiate_ty_bounds(&lambda.sig.bounds, RegistrationMode::Normal)
            .map_err(|e| {
                self.pop_append_errs();
                e
            })?;
        self.pop_append_errs();
        let t = if is_procedural {
            Type::proc(non_default_params, default_params, body.t())
        } else {
            Type::func(non_default_params, default_params, body.t())
        };
        let t = if bounds.is_empty() {
            t
        } else {
            Type::quantified(t, bounds)
        };
        Ok(hir::Lambda::new(id, lambda.sig.params, lambda.op, body, t))
    }

    fn lower_def(&mut self, def: ast::Def) -> LowerResult<hir::Def> {
        log!("[DEBUG] entered {}({})", fn_name!(), def.sig);
        // FIXME: Instant
        self.ctx
            .grow(def.sig.name_as_str(), ContextKind::Instant, Private)?;
        let res = match def.sig {
            ast::Signature::Subr(sig) => self.lower_subr_def(sig, def.body),
            ast::Signature::Var(sig) => self.lower_var_def(sig, def.body),
        };
        // TODO: Context上の関数に型境界情報を追加
        self.pop_append_errs();
        res
    }

    fn lower_var_def(
        &mut self,
        sig: ast::VarSignature,
        body: ast::DefBody,
    ) -> LowerResult<hir::Def> {
        log!("[DEBUG] entered {}({sig})", fn_name!());
        self.ctx.preregister(body.block.ref_payload())?;
        let block = self.lower_block(body.block)?;
        let found_body_t = block.ref_t();
        let opt_expect_body_t = self
            .ctx
            .outer
            .as_ref()
            .unwrap()
            .get_current_scope_var(sig.inspect().unwrap())
            .map(|vi| vi.t.clone());
        let name = sig.pat.inspect().unwrap();
        if let Some(expect_body_t) = opt_expect_body_t {
            if let Err(e) = self.return_t_check(sig.loc(), name, &expect_body_t, found_body_t) {
                self.errs.push(e);
            }
        }
        let id = body.id;
        // TODO: cover all VarPatterns
        self.ctx
            .outer
            .as_mut()
            .unwrap()
            .assign_var(&sig, id, found_body_t)?;
        match block.first().unwrap() {
            hir::Expr::Call(call) => {
                if let ast::VarPattern::VarName(name) = &sig.pat {
                    if call.is_import_call() {
                        self.ctx
                            .outer
                            .as_mut()
                            .unwrap()
                            .import_mod(name, &call.args.pos_args.first().unwrap().expr)?;
                    }
                } else {
                    todo!()
                }
            }
            _other => {}
        }
        let sig = hir::VarSignature::new(sig.pat, found_body_t.clone());
        let body = hir::DefBody::new(body.op, block, body.id);
        Ok(hir::Def::new(hir::Signature::Var(sig), body))
    }

    // NOTE: 呼ばれている間はinner scopeなので注意
    fn lower_subr_def(
        &mut self,
        sig: ast::SubrSignature,
        body: ast::DefBody,
    ) -> LowerResult<hir::Def> {
        log!("[DEBUG] entered {}({sig})", fn_name!());
        let t = self
            .ctx
            .outer
            .as_ref()
            .unwrap()
            .get_current_scope_var(sig.name.inspect())
            .unwrap_or_else(|| {
                log!("{}\n", sig.name.inspect());
                log!("{}\n", self.ctx.outer.as_ref().unwrap());
                panic!()
            }) // FIXME: or instantiate
            .t
            .clone();
        self.ctx.assign_params(&sig.params, None)?;
        self.ctx.preregister(body.block.ref_payload())?;
        let block = self.lower_block(body.block)?;
        let found_body_t = block.ref_t();
        let expect_body_t = t.return_t().unwrap();
        if let Err(e) =
            self.return_t_check(sig.loc(), sig.name.inspect(), expect_body_t, found_body_t)
        {
            self.errs.push(e);
        }
        let id = body.id;
        self.ctx
            .outer
            .as_mut()
            .unwrap()
            .assign_subr(&sig, id, found_body_t)?;
        let sig = hir::SubrSignature::new(sig.name, sig.params, t);
        let body = hir::DefBody::new(body.op, block, body.id);
        Ok(hir::Def::new(hir::Signature::Subr(sig), body))
    }

    // Call.obj == Accessor cannot be type inferred by itself (it can only be inferred with arguments)
    // so turn off type checking (check=false)
    fn lower_expr(&mut self, expr: ast::Expr, check: bool) -> LowerResult<hir::Expr> {
        log!("[DEBUG] entered {}", fn_name!());
        match expr {
            ast::Expr::Lit(lit) => Ok(hir::Expr::Lit(hir::Literal::from(lit.token))),
            ast::Expr::Array(arr) => Ok(hir::Expr::Array(self.lower_array(arr, check)?)),
            ast::Expr::Accessor(acc) => Ok(hir::Expr::Accessor(self.lower_acc(acc, check)?)),
            ast::Expr::BinOp(bin) => Ok(hir::Expr::BinOp(self.lower_bin(bin)?)),
            ast::Expr::UnaryOp(unary) => Ok(hir::Expr::UnaryOp(self.lower_unary(unary)?)),
            ast::Expr::Call(call) => Ok(hir::Expr::Call(self.lower_call(call)?)),
            ast::Expr::Lambda(lambda) => Ok(hir::Expr::Lambda(self.lower_lambda(lambda)?)),
            ast::Expr::Def(def) => Ok(hir::Expr::Def(self.lower_def(def)?)),
            other => todo!("{other}"),
        }
    }

    fn lower_block(&mut self, ast_block: ast::Block) -> LowerResult<hir::Block> {
        log!("[DEBUG] entered {}", fn_name!());
        let mut hir_block = Vec::with_capacity(ast_block.len());
        for expr in ast_block.into_iter() {
            let expr = self.lower_expr(expr, true)?;
            hir_block.push(expr);
        }
        Ok(hir::Block::new(hir_block))
    }

    pub fn lower(&mut self, ast: AST, mode: &str) -> Result<(HIR, LowerWarnings), LowerErrors> {
        log!("{GREEN}[DEBUG] the type-checking process has started.");
        let mut module = hir::Module::with_capacity(ast.module.len());
        self.ctx.preregister(ast.module.ref_payload())?;
        for expr in ast.module.into_iter() {
            match self
                .lower_expr(expr, true)
                .and_then(|e| self.use_check(e, mode))
            {
                Ok(expr) => {
                    module.push(expr);
                }
                Err(e) => {
                    self.errs.push(e);
                }
            }
        }
        let hir = HIR::new(ast.name, module);
        log!("HIR (not derefed):\n{hir}");
        let hir = self.ctx.deref_toplevel(hir)?;
        log!(
            "[DEBUG] {}() has completed, found errors: {}",
            fn_name!(),
            self.errs.len()
        );
        if self.errs.is_empty() {
            log!("HIR:\n{hir}");
            log!("[DEBUG] the type-checking process has completed.{RESET}");
            Ok((hir, LowerWarnings::from(self.warns.take_all())))
        } else {
            log!("{RED}[DEBUG] the type-checking process has failed.{RESET}");
            Err(LowerErrors::from(self.errs.take_all()))
        }
    }
}

impl Default for ASTLowerer {
    fn default() -> Self {
        Self::new()
    }
}
