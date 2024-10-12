use syn::File;

use syn::Item;

pub fn Parse_string(Str: &str) {
    let File: syn::File = syn::parse_str(Str).expect("Error parsing lvgl bindings");
    Parse_file(File);
}

pub fn Parse_file(File: File) {
    for Item in File.items {
        Parse_item(Item);
    }
}

pub fn Parse_item(Item: Item) {
    match Item {
        syn::Item::Fn(func) => {
            println!("Found a function: {}", func.sig.ident);
        }
        syn::Item::Struct(strct) => {
            println!("Found a struct: {}", strct.ident);
        }
        syn::Item::Enum(enm) => {
            println!("Found an enum: {}", enm.ident);
        }
        syn::Item::Mod(md) => {
            println!("Found a module: {}", md.ident);
        }
        syn::Item::Type(ty) => {
            println!("Found a type: {}", ty.ident);
        }
        syn::Item::Const(cnst) => {
            println!("Found a const: {}", cnst.ident);
        }
        syn::Item::Static(sttc) => {
            println!("Found a static: {}", sttc.ident);
        }
        syn::Item::Trait(trt) => {
            println!("Found a trait: {}", trt.ident);
        }

        syn::Item::Use(use_) => {
            println!("Found a use");
        }
        syn::Item::ExternCrate(extern_crate) => {
            println!("Found an extern crate: {}", extern_crate.ident);
        }
        syn::Item::ForeignMod(foreign_mod) => {
            foreign_mod.items.into_iter().for_each(Parse_foreign_item);
        }
        syn::Item::Verbatim(verbatim) => {
            println!("Found a verbatim: {}", verbatim);
        }
        _ => {}
    }
}

pub fn Parse_foreign_item(foreign_item: syn::ForeignItem) {
    match foreign_item {
        syn::ForeignItem::Fn(func) => {
            println!("Found a foreign function: {}", func.sig.ident);
        }
        syn::ForeignItem::Static(sttc) => {
            println!("Found a foreign static: {}", sttc.ident);
        }
        syn::ForeignItem::Type(ty) => {
            println!("Found a foreign type: {}", ty.ident);
        }
        _ => {}
    }
}
