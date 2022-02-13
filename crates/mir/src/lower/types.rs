// TODO: remove this.
#![allow(unused)]

use std::rc::Rc;

use crate::{
    db::MirDb,
    ir::{
        types::{ArrayDef, MapDef, StructDef, TupleDef},
        Type, TypeId,
    },
};

use fe_analyzer::namespace::types as analyzer_types;

pub fn lower_type(db: &dyn MirDb, ty: &analyzer_types::Type) -> TypeId {
    match ty {
        analyzer_types::Type::Base(base) => lower_base(db, base),
        analyzer_types::Type::Array(arr) => lower_array(db, arr),
        analyzer_types::Type::Map(map) => lower_map(db, map),
        analyzer_types::Type::Tuple(tup) => lower_tuple(db, tup),
        analyzer_types::Type::String(string) => lower_string(db, string),
        analyzer_types::Type::Contract(contract) => lower_contract(db, contract),
        analyzer_types::Type::SelfContract(contract) => lower_contract(db, contract),
        analyzer_types::Type::Struct(struct_) => lower_struct(db, struct_),
    }
}

fn lower_base(db: &dyn MirDb, base: &analyzer_types::Base) -> TypeId {
    use analyzer_types::{Base, Integer};

    let ty = match base {
        Base::Numeric(int_ty) => match int_ty {
            Integer::I8 => Type::I8,
            Integer::I16 => Type::I16,
            Integer::I32 => Type::I32,
            Integer::I64 => Type::I64,
            Integer::I128 => Type::I128,
            Integer::I256 => Type::I256,
            Integer::U8 => Type::U8,
            Integer::U16 => Type::U16,
            Integer::U32 => Type::U32,
            Integer::U64 => Type::U64,
            Integer::U128 => Type::U128,
            Integer::U256 => Type::U256,
        },

        Base::Bool => Type::Bool,
        Base::Address => Type::Address,
        Base::Unit => Type::Unit,
    };

    intern_type(db, ty)
}

fn lower_array(db: &dyn MirDb, arr: &analyzer_types::Array) -> TypeId {
    let len = arr.size;
    let elem_ty = lower_base(db, &arr.inner);

    let def = ArrayDef { elem_ty, len };
    let ty = Type::Array(def);
    intern_type(db, ty)
}

fn lower_map(db: &dyn MirDb, map: &analyzer_types::Map) -> TypeId {
    let key_ty = lower_base(db, &map.key);
    let value_ty = db.lowered_type(*map.value.clone());

    let def = MapDef { key_ty, value_ty };
    let ty = Type::Map(def);
    intern_type(db, ty)
}

fn lower_tuple(db: &dyn MirDb, tup: &analyzer_types::Tuple) -> TypeId {
    let items = tup
        .items
        .iter()
        .map(|item| lower_fixed_size(db, item))
        .collect();

    let def = TupleDef { items };
    let ty = Type::Tuple(def);
    intern_type(db, ty)
}

/// `FeString` type is lowered into Array<u8> type.
fn lower_string(db: &dyn MirDb, string: &analyzer_types::FeString) -> TypeId {
    // We assume a string consists of only ascii encoding chars.
    let elem_ty = intern_type(db, Type::U8);
    let len = string.max_size;

    let def = ArrayDef { elem_ty, len };
    let ty = Type::Array(def);
    intern_type(db, ty)
}

fn lower_contract(db: &dyn MirDb, contract: &analyzer_types::Contract) -> TypeId {
    let id = contract.id;

    let name = id.name(db.upcast());

    // Lower contract fields.
    let fields = id
        .fields(db.upcast())
        .iter()
        .map(|(fname, fid)| {
            let analyzer_types = fid.typ(db.upcast()).unwrap();
            let ty = db.lowered_type(analyzer_types);
            (fname.clone(), ty)
        })
        .collect();

    // Obtain span.
    let span = id.span(db.upcast());

    // Lower module.
    let analyzer_module_id = id.module(db.upcast());
    let module_id = db.lowered_module(analyzer_module_id);

    let def = StructDef {
        name,
        fields,
        span,
        module_id,
    };
    let ty = Type::Contract(def);
    intern_type(db, ty)
}

fn lower_struct(db: &dyn MirDb, struct_: &analyzer_types::Struct) -> TypeId {
    let id = struct_.id;

    let name = id.name(db.upcast());

    // Lower struct fields.
    let fields = id
        .fields(db.upcast())
        .iter()
        .map(|(fname, fid)| {
            let analyzer_types = fid.typ(db.upcast()).unwrap();
            let ty = lower_fixed_size(db, &analyzer_types);
            (fname.clone(), ty)
        })
        .collect();

    // obtain span.
    let span = id.span(db.upcast());

    // Lower module.
    let analyzer_module_id = id.module(db.upcast());
    let module_id = db.lowered_module(analyzer_module_id);

    let def = StructDef {
        name,
        fields,
        span,
        module_id,
    };
    let ty = Type::Struct(def);
    intern_type(db, ty)
}

fn intern_type(db: &dyn MirDb, ty: Type) -> TypeId {
    db.intern_type(Rc::new(ty))
}

fn lower_fixed_size(db: &dyn MirDb, fixed_size: &analyzer_types::FixedSize) -> TypeId {
    use analyzer_types::FixedSize;

    match fixed_size {
        FixedSize::Base(base) => lower_base(db, base),
        FixedSize::Array(arr) => lower_array(db, arr),
        FixedSize::Tuple(tup) => lower_tuple(db, tup),
        FixedSize::String(string) => lower_string(db, string),
        FixedSize::Contract(contract) => lower_contract(db, contract),
        FixedSize::Struct(struct_) => lower_struct(db, struct_),
    }
}