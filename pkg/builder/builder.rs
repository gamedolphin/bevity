use genco::prelude::*;

pub fn build(file_path: &str, output_path: &str) -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed={}", file_path);
    println!("cargo:rerun-if-changed=build.rs");

    std::fs::create_dir_all(output_path)?;

    let file = std::fs::read_to_string(file_path)?;
    let syntax = syn::parse_file(&file)?;

    syntax
        .items
        .iter()
        .filter_map(|s| match s {
            syn::Item::Struct(s) => Some(s),
            _ => None,
        })
        .filter(|item| {
            item.attrs
                .iter()
                .any(|attr| attr.path().is_ident("derive") && attribute_is_component(attr))
        })
        .map(|strukt| {
            let name = &strukt.ident.to_string();
            let class_name = quote! { $name };
            let fields = strukt.fields.iter().map(|field| -> csharp::Tokens {
                let field_name = &field.ident.clone().unwrap().to_string();
                let field_name = quote! { $field_name };
                let field_type = map_type_to_csharp(&field.ty);

                quote! {
                    public $field_type $field_name;
                }
            });

            let tokens: csharp::Tokens = quote! {
                using UnityEngine;

                public class $class_name: MonoBehaviour
                {
                    $(for field in fields => $field)
                }
            };

            (name.clone(), tokens)
        })
        .try_for_each(|(name, contents)| {
            std::fs::write(
                format!("{}/{}.cs", output_path, name),
                contents.to_file_string().unwrap(),
            )
        })?;

    Ok(())
}

fn attribute_is_component(attr: &syn::Attribute) -> bool {
    match &attr.meta {
        syn::Meta::List(list) => list.tokens.to_string().contains("Component"),
        _ => false,
    }
}

fn map_type_to_csharp(ty: &syn::Type) -> String {
    let syn::Type::Path(path) = ty else {
        panic!("expected rust path type");
    };

    let path = path
        .path
        .get_ident()
        .expect("expected simple path type")
        .to_string();

    match path.as_str() {
        "f32" => "float",
        "f64" => "double",
        "i32" => "int",
        "i64" => "long",
        "u32" => "uint",
        "u64" => "ulong",
        "String" => "string",
        _ => panic!("unsupported base type"),
    }
    .to_string()
}
