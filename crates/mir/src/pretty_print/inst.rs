use std::io::{self, Write};

use crate::{
    db::MirDb,
    ir::{function::BodyDataStore, inst::InstKind, InstId},
};

use super::PrettyPrint;

impl PrettyPrint for InstId {
    fn pretty_print<W: Write>(
        &self,
        db: &dyn MirDb,
        store: &BodyDataStore,
        w: &mut W,
    ) -> io::Result<()> {
        if let Some(result) = store.inst_result(*self) {
            result.pretty_print(db, store, w)?;
            write!(w, ": ")?;
            store.value_ty(result).pretty_print(db, store, w)?;
            write!(w, " = ")?;
        }

        match &store.inst_data(*self).kind {
            InstKind::Declare { local } => {
                write!(w, "let ")?;
                local.pretty_print(db, store, w)
            }

            InstKind::Assign { lhs, rhs } => {
                lhs.pretty_print(db, store, w)?;
                write!(w, ": ")?;
                store.value_ty(*lhs).pretty_print(db, store, w)?;
                write!(w, " = ")?;
                rhs.pretty_print(db, store, w)
            }

            InstKind::Unary { op, value } => {
                write!(w, "{}", op)?;
                value.pretty_print(db, store, w)
            }

            InstKind::Binary { op, lhs, rhs } => {
                lhs.pretty_print(db, store, w)?;
                write!(w, " {} ", op)?;
                rhs.pretty_print(db, store, w)
            }

            InstKind::Cast { value, to } => {
                value.pretty_print(db, store, w)?;
                write!(w, " as ")?;
                to.pretty_print(db, store, w)
            }

            InstKind::AggregateConstruct { ty, args } => {
                ty.pretty_print(db, store, w)?;
                write!(w, "{{")?;
                if args.is_empty() {
                    return write!(w, "}}");
                }

                let arg_len = args.len();
                for (arg_idx, arg) in args.iter().enumerate().take(arg_len - 1) {
                    write!(w, "{}: ", arg_idx)?;
                    arg.pretty_print(db, store, w)?;
                    write!(w, ", ")?;
                }
                let arg = args[arg_len - 1];
                write!(w, "{}: ", arg_len)?;
                arg.pretty_print(db, store, w)?;
                write!(w, "}}")
            }

            InstKind::AggregateAccess { value, index } => {
                value.pretty_print(db, store, w)?;
                write!(w, ".")?;
                index.pretty_print(db, store, w)
            }

            InstKind::MapAccess { value, key } => {
                value.pretty_print(db, store, w)?;
                write!(w, "{{")?;
                key.pretty_print(db, store, w)?;
                write!(w, "}}")
            }

            InstKind::Call {
                func,
                args,
                call_type,
            } => {
                let name = func.name_with_class(db);
                write!(w, "{}@{}(", name, call_type)?;
                args.as_slice().pretty_print(db, store, w)?;
                write!(w, ")")
            }

            InstKind::Jump { dest } => {
                write!(w, "jump bb{}", dest.index())
            }

            InstKind::Branch { cond, then, else_ } => {
                write!(w, "branch ")?;
                cond.pretty_print(db, store, w)?;
                write!(w, " then: bb{} else: bb{}", then.index(), else_.index())
            }

            InstKind::Revert { arg } => {
                write!(w, "revert ")?;
                arg.pretty_print(db, store, w)
            }

            InstKind::Emit { arg } => {
                write!(w, "emit ")?;
                arg.pretty_print(db, store, w)
            }

            InstKind::Return { arg } => {
                write!(w, "return ")?;
                arg.pretty_print(db, store, w)
            }

            InstKind::Intrinsic { op, args } => {
                write!(w, "{}(", op)?;
                args.as_slice().pretty_print(db, store, w)?;
                write!(w, "{})", op)
            }
        }
    }
}