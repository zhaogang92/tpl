///! Type system
use super::parser::{self, Term, Type, VarContext,  Context, Bind};

pub fn type_of(t: &Term, ctx: &mut Context) -> Type {
    match t {
        Term::True(_) | Term::False(_) => Type::TyBool,
        Term::Var(_, index, _) => {
            if let Some(VarContext(x, bind)) = ctx.get(*index as usize) {
                if let Bind::VarBind(ty) = bind {
                    ty.clone()
                } else {
                    panic!("No type info for variable: {}", x);
                }
            } else {
                panic!("Context not found for variable index: {}", index);
            }
        }
        Term::If(_, cond, body, alt) => {
            let ty_t1 = type_of(cond, ctx);
            if ty_t1 == Type::TyBool {
                let ty_t2 = type_of(body, ctx);
                let ty_t3 = type_of(alt, ctx);
                if ty_t2 == ty_t3 {
                    ty_t2.clone()
                } else {
                    panic!("The type of two if branches must match, {:?} != {:?}", ty_t1, ty_t2)
                }
            } else {
                panic!("If condition must be boolean, got: {:?}", ty_t1)
            }
        }
        Term::App(info, f, arg) => {
            let ty_t1 = type_of(f, ctx);
            let ty_t2 = type_of(arg, ctx);
            match ty_t1 {
                Type::TyArr(ty_t11, ty_t12) => {
                    if ty_t2 == *ty_t11 {
                        *ty_t12
                    } else {
                        panic!("parameter mismatch: {:?}", info)
                    }
                }
                _ => panic!("Expect TyArr, get: {:}", ty_t1)
            }
        }
        Term::Abst(_, x, ty_t1, t2) => {
            parser::add_bind(ctx, x, Bind::VarBind(ty_t1.clone()));
            let ty_t2 = type_of(t2, ctx);
            Type::TyArr(Box::new(ty_t1.clone()), Box::new(ty_t2))
        }
        _ => unimplemented!("Unimplemente for term: {:#?}", t)
    }
}