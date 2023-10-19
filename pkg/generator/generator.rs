use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Ident};

#[proc_macro]
pub fn inbuilt_component_list(item: TokenStream) -> TokenStream {
    // (GameObject -> UnityGameObject)
    let tuples = parse_macro_input!(item as syn::TypeTuple);

    let filtered = tuples
        .elems
        .iter()
        .filter_map(|v| match v {
            syn::Type::Path(p) => Some(p),
            _ => None,
        })
        .filter_map(|p| p.path.get_ident());

    let components = filtered.clone().map(|ident| {
        let ident = format_ident!("Unity{}", ident);
        let (meta_struct_name, dirty_struct_name, _) = get_fn_idents(&ident);

        quote! {
            #[derive(bevy::prelude::Component, Debug, Clone)]
            pub struct #meta_struct_name {
                pub object_id: i64,
            }

            #[derive(bevy::prelude::Component)]
            pub struct #dirty_struct_name;
        }
    });

    let enums = filtered.clone().map(|ident| {
        let inbuilt_ident = format_ident!("Unity{}", ident);
        quote! {
            #ident(#inbuilt_ident)
        }
    });

    let change_enums = enums.clone();

    let meta_insert = filtered.clone().map(|ident| {
        let inbuilt_ident = format_ident!("Unity{}", ident);
        let (meta_struct_name, _, _) = get_fn_idents(&inbuilt_ident);

        quote! {
            UnitySceneObject::#ident(_) => { commands.insert(#meta_struct_name { object_id }); }
        }
    });

    quote! {
        #(#components)*

        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[serde(tag = "object_type")]
        pub enum UnitySceneObject<T> {
            #(#enums),*
            ,MonoBehaviour(T),

            #[serde(other)]
            DontCare,
        }

        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub enum UnityChangeObject<T> {
            #(#change_enums),*
            ,MonoBehaviour(T),

            #[serde(other)]
            DontCare,
        }

        impl<T> UnitySceneObject<T> {
            pub fn spawn_meta(&self, object_id: i64, commands: &mut bevy::ecs::system::EntityCommands) {
                match self {
                    UnitySceneObject::MonoBehaviour(T) => {},
                    UnitySceneObject::DontCare => {},
                    #(#meta_insert,)*
                };
            }
        }
    }
    .into()
}

fn get_fn_idents(ident: &Ident) -> (Ident, Ident, Ident) {
    let meta_struct_name = format_ident!("{}Meta", ident);
    let dirty_struct_name = format_ident!("{}Dirty", ident);
    let track_fn_name = format_ident!("track_{}", ident);

    (meta_struct_name, dirty_struct_name, track_fn_name)
}

#[proc_macro]
pub fn exported_component_list(item: TokenStream) -> TokenStream {
    let tuples = parse_macro_input!(item as syn::TypeTuple);

    let filtered = tuples
        .elems
        .iter()
        .filter_map(|v| match v {
            syn::Type::Path(p) => Some(p),
            _ => None,
        })
        .filter_map(|p| p.path.get_ident());

    let generated = filtered.clone().map(|ident| {
        let (meta_struct_name, dirty_struct_name, track_fn_name) = get_fn_idents(ident);

        quote! {
            #[derive(Component, Debug, Clone)]
            pub struct #meta_struct_name {
                pub object_id: i64,
            }

            #[derive(Component)]
            pub struct #dirty_struct_name;

            #[allow(non_snake_case)]
            pub fn #track_fn_name(query: Query<(Entity, &#meta_struct_name, &#ident, Option<&#dirty_struct_name>), Changed<#ident>>,
                mut change_map: ResMut<bevity::UnityChangeMap>,
                mut commands: Commands,) {
                    for (entity, meta, val, dirty) in &query {
                        if let Ok(serialized) =
                        serde_json::to_string(&bevity::UnitySceneObject::<BevityExported>::MonoBehaviour(
                            BevityExported::#ident(val.clone()),
                        ))
                        {
                            if dirty.is_some() {
                                change_map.dirty.insert(meta.object_id);
                                commands.entity(entity).remove::<#dirty_struct_name>();
                            } else {
                                change_map.changes.insert(meta.object_id, serialized);
                            }
                        };
                    }
                }

        }
    });

    let exported = filtered.clone().map(|ident| quote! { #ident(#ident) });

    let trackers = filtered.clone().map(|ident| {
        let fn_name = format_ident!("track_{}", ident);
        quote! { #fn_name }
    });

    let add_components = filtered.clone().map(|ident| {
        let (meta_comp_name, _, _) = get_fn_idents(ident);
        let local_name = format_ident!("{}_obj", ident);
        quote! {
            #[allow(non_snake_case)]
            BevityExported::#ident(#local_name) => {
                cmd.insert(#local_name.clone());
                cmd.insert(#meta_comp_name { object_id });
            }
        }
    });

    let update_components = filtered.clone().map(|ident| {
        let local_name = format_ident!("{}_upd", ident);
        let dirty_name = format_ident!("{}Dirty", ident);
        quote! {
            #[allow(non_snake_case)]
            BevityExported::#ident(#local_name) => {
                cmd.insert(#local_name.clone());
                cmd.insert(#dirty_name);
            }
        }
    });

    quote! {
        #(#generated)*

        #[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
        #[serde(untagged)]
        pub enum BevityExported {
            #[default]
            DontCare,

            #(#exported),*
        }

        impl Plugin for BevityExported {
            fn build(&self, app: &mut App) {
                if std::env::var_os(bevity::ENABLE_BEVITY_EDITOR).is_none() {
                    return;
                }

                app.add_systems(Update, ( #(#trackers),* ));
            }
        }

        impl bevity::MonoBehaviour for BevityExported {
            fn add_component_to_entity(&self, object_id: i64, cmd: &mut bevy::ecs::system::EntityCommands) {
                match self {
                    #(#add_components)*

                    BevityExported::DontCare => {
                        println!("found dont care component");
                    }
                };
            }

            fn update_component(&self, cmd: &mut bevy::ecs::world::EntityMut) {
                match self {
                    #(#update_components)*

                    BevityExported::DontCare => {
                        println!("found dont care component");
                    }
                };
            }
        }

    }
    .into()
}
