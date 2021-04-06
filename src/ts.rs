///! Type system
use std::collections::HashMap;
use super::parser::{self, Term, Type, VarContext,  Context, Bind};

pub fn type_of(t: &Term, ctx: &mut Context) -> Type {
    match t {
        Term::True(_) | Term::False(_) => Type::TyBool,
        Term::Record(_, elements) => {
            let mut tys = HashMap::new();
            for (name, box ele_t) in elements {
                tys.insert(name.to_string(), Box::new(type_of(ele_t, ctx)));
            }
            Type::TyRecord(tys)
        }
        Term::Proj(_, box proj_t, name) => {
            if let Type::TyRecord(ty) = type_of(proj_t, ctx) {
                if let Some(box ele_ty) = &ty.get(name) {
                    ele_ty.clone()
                } else {
                    panic!("Failed to find key: {} in the record", name)
                }
            } else {
                panic!("Expect record type for term: {:?}", proj_t)
            }
        }
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
        // TODO: compute join type
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
            match &ty_t1 {
                Type::TyArr(box ty_t11, box ty_t12) => {
                    if sub_type(&ty_t2, ty_t11) {
                        ty_t12.clone()
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


// Subtyping


pub fn sub_type(s: &Type, t: &Type) -> bool {
    if *s == *t {
        return true;
    }
    match (s, t) {
        (Type::TyRecord(sl), Type::TyRecord(tl)) => {
            for (tl_name, box tl_ty) in tl {
                if let Some(box sl_ty) = sl.get(tl_name) {
                    if !sub_type(sl_ty, tl_ty) {
                        return false;
                    }
                } else {
                    return false;   // element of t must be in subtype s
                }
            }
            true
        }
        (_, Type::TyTop) => true,
        (Type::TyArr(box s1, box s2), Type::TyArr(box t1, box t2)) => {
            sub_type(t1, s1) && sub_type(s2, t2)
        }
        _ => false,
    }
}
