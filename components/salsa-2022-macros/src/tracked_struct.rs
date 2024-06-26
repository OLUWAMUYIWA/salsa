use proc_macro2::{Literal, Span, TokenStream};

use crate::salsa_struct::{SalsaField, SalsaStruct};

/// For an tracked struct `Foo` with fields `f1: T1, ..., fN: TN`, we generate...
///
/// * the "id struct" `struct Foo(salsa::Id)`
/// * the tracked ingredient, which maps the id fields to the `Id`
/// * for each value field, a function ingredient
pub(crate) fn tracked(
    args: proc_macro::TokenStream,
    struct_item: syn::ItemStruct,
) -> syn::Result<TokenStream> {
    SalsaStruct::with_struct(args, struct_item).and_then(|el| TrackedStruct(el).generate_tracked())
}

struct TrackedStruct(SalsaStruct<Self>);

impl std::ops::Deref for TrackedStruct {
    type Target = SalsaStruct<Self>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl crate::options::AllowedOptions for TrackedStruct {
    const RETURN_REF: bool = false;

    const SPECIFY: bool = false;

    const NO_EQ: bool = false;

    const SINGLETON: bool = false;

    const JAR: bool = true;

    const DATA: bool = true;

    const DB: bool = false;

    const RECOVERY_FN: bool = false;

    const LRU: bool = false;

    const CONSTRUCTOR_NAME: bool = true;
}

impl TrackedStruct {
    fn generate_tracked(&self) -> syn::Result<TokenStream> {
        self.validate_tracked()?;

        let id_struct = self.id_struct();
        let config_struct = self.config_struct();
        let config_impl = self.config_impl(&config_struct);
        let inherent_impl = self.tracked_inherent_impl();
        let ingredients_for_impl = self.tracked_struct_ingredients(&config_struct);
        let salsa_struct_in_db_impl = self.salsa_struct_in_db_impl();
        let tracked_struct_in_db_impl = self.tracked_struct_in_db_impl();
        let as_id_impl = self.as_id_impl();
        let as_debug_with_db_impl = self.as_debug_with_db_impl();
        Ok(quote! {
            #config_struct
            #config_impl
            #id_struct
            #inherent_impl
            #ingredients_for_impl
            #salsa_struct_in_db_impl
            #tracked_struct_in_db_impl
            #as_id_impl
            #as_debug_with_db_impl
        })
    }

    fn validate_tracked(&self) -> syn::Result<()> {
        Ok(())
    }

    fn config_struct(&self) -> syn::ItemStruct {
        let config_ident = syn::Ident::new(
            &format!("__{}Config", self.id_ident()),
            self.id_ident().span(),
        );
        let visibility = self.visibility();

        parse_quote! {
            #visibility struct #config_ident {
                _uninhabited: std::convert::Infallible,
            }
        }
    }

    fn config_impl(&self, config_struct: &syn::ItemStruct) -> syn::ItemImpl {
        let id_ident = self.id_ident();
        let config_ident = &config_struct.ident;
        let field_tys: Vec<_> = self.all_fields().map(SalsaField::ty).collect();
        let id_field_indices = self.id_field_indices();
        let arity = self.all_field_count();

        // Create the function body that will update the revisions for each field.
        // If a field is a "backdate field" (the default), then we first check if
        // the new value is `==` to the old value. If so, we leave the revision unchanged.
        let old_value = syn::Ident::new("old_value_", Span::call_site());
        let new_value = syn::Ident::new("new_value_", Span::call_site());
        let revisions = syn::Ident::new("revisions_", Span::call_site());
        let current_revision = syn::Ident::new("current_revision_", Span::call_site());
        let update_revisions: TokenStream = self
            .all_fields()
            .zip(0..)
            .map(|(field, i)| {
                let field_index = Literal::u32_unsuffixed(i);
                if field.is_backdate_field() {
                    quote_spanned! { field.span() =>
                        if #old_value.#field_index != #new_value.#field_index {
                            #revisions[#field_index] = #current_revision;
                        }
                    }
                } else {
                    quote_spanned! { field.span() =>
                        #revisions[#field_index] = #current_revision;
                    }
                }
            })
            .collect();

        parse_quote! {
            impl salsa::tracked_struct::Configuration for #config_ident {
                type Id = #id_ident;
                type Fields = ( #(#field_tys,)* );
                type Revisions = [salsa::Revision; #arity];

                #[allow(clippy::unused_unit)]
                fn id_fields(fields: &Self::Fields) -> impl std::hash::Hash {
                    ( #( &fields.#id_field_indices ),* )
                }

                fn revision(revisions: &Self::Revisions, field_index: u32) -> salsa::Revision {
                    revisions[field_index as usize]
                }

                fn new_revisions(current_revision: salsa::Revision) -> Self::Revisions {
                    [current_revision; #arity]
                }

                fn update_revisions(
                    #current_revision: salsa::Revision,
                    #old_value: &Self::Fields,
                    #new_value: &Self::Fields,
                    #revisions: &mut Self::Revisions,
                ) {
                    #update_revisions
                }
            }
        }
    }

    /// Generate an inherent impl with methods on the tracked type.
    fn tracked_inherent_impl(&self) -> syn::ItemImpl {
        let ident = self.id_ident();
        let jar_ty = self.jar_ty();
        let db_dyn_ty = self.db_dyn_ty();
        let tracked_field_ingredients: Literal = self.tracked_field_ingredients_index();

        let field_indices = self.all_field_indices();
        let field_vises: Vec<_> = self.all_fields().map(SalsaField::vis).collect();
        let field_tys: Vec<_> = self.all_fields().map(SalsaField::ty).collect();
        let field_get_names: Vec<_> = self.all_fields().map(SalsaField::get_name).collect();
        let field_clones: Vec<_> = self.all_fields().map(SalsaField::is_clone_field).collect();
        let field_getters: Vec<syn::ImplItemMethod> = field_indices.iter().zip(&field_get_names).zip(&field_tys).zip(&field_vises).zip(&field_clones).map(|((((field_index, field_get_name), field_ty), field_vis), is_clone_field)|
            if !*is_clone_field {
                parse_quote_spanned! { field_get_name.span() =>
                    #field_vis fn #field_get_name<'db>(self, __db: &'db #db_dyn_ty) -> &'db #field_ty
                    {
                        let (__jar, __runtime) = <_ as salsa::storage::HasJar<#jar_ty>>::jar(__db);
                        let __ingredients = <#jar_ty as salsa::storage::HasIngredientsFor< #ident >>::ingredient(__jar);
                        &__ingredients.#tracked_field_ingredients[#field_index].field(__runtime, self).#field_index
                    }
                }
            } else {
                parse_quote_spanned! { field_get_name.span() =>
                    #field_vis fn #field_get_name<'db>(self, __db: &'db #db_dyn_ty) -> #field_ty
                    {
                        let (__jar, __runtime) = <_ as salsa::storage::HasJar<#jar_ty>>::jar(__db);
                        let __ingredients = <#jar_ty as salsa::storage::HasIngredientsFor< #ident >>::ingredient(__jar);
                        __ingredients.#tracked_field_ingredients[#field_index].field(__runtime, self).#field_index.clone()
                    }
                }
            }
        )
        .collect();

        let field_names = self.all_field_names();
        let field_tys = self.all_field_tys();
        let constructor_name = self.constructor_name();

        parse_quote! {
            #[allow(dead_code, clippy::pedantic, clippy::complexity, clippy::style)]
            impl #ident {
                pub fn #constructor_name(__db: &#db_dyn_ty, #(#field_names: #field_tys,)*) -> Self
                {
                    let (__jar, __runtime) = <_ as salsa::storage::HasJar<#jar_ty>>::jar(__db);
                    let __ingredients = <#jar_ty as salsa::storage::HasIngredientsFor< #ident >>::ingredient(__jar);
                    let __id = __ingredients.0.new_struct(
                        __runtime,
                        (#(#field_names,)*),
                    );
                    __id
                }

                #(#field_getters)*
            }
        }
    }

    /// Generate the `IngredientsFor` impl for this tracked struct.
    ///
    /// The tracked struct's ingredients include both the main tracked struct ingredient along with a
    /// function ingredient for each of the value fields.
    fn tracked_struct_ingredients(&self, config_struct: &syn::ItemStruct) -> syn::ItemImpl {
        use crate::literal;
        let ident = self.id_ident();
        let jar_ty = self.jar_ty();
        let config_struct_name = &config_struct.ident;
        let field_indices: Vec<Literal> = self.all_field_indices();
        let arity = self.all_field_count();
        let tracked_struct_ingredient: Literal = self.tracked_struct_ingredient_index();
        let tracked_fields_ingredients: Literal = self.tracked_field_ingredients_index();
        let debug_name_struct = literal(self.id_ident());
        let debug_name_fields: Vec<_> = self.all_field_names().into_iter().map(literal).collect();

        parse_quote! {
            impl salsa::storage::IngredientsFor for #ident {
                type Jar = #jar_ty;
                type Ingredients = (
                    salsa::tracked_struct::TrackedStructIngredient<#config_struct_name>,
                    [salsa::tracked_struct::TrackedFieldIngredient<#config_struct_name>; #arity],
                );

                fn create_ingredients<DB>(
                    routes: &mut salsa::routes::Routes<DB>,
                ) -> Self::Ingredients
                where
                    DB: salsa::DbWithJar<Self::Jar> + salsa::storage::JarFromJars<Self::Jar>,
                {
                    let struct_ingredient =                         {
                        let index = routes.push(
                            |jars| {
                                let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                                let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar);
                                &ingredients.#tracked_struct_ingredient
                            },
                            |jars| {
                                let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars_mut(jars);
                                let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient_mut(jar);
                                &mut ingredients.#tracked_struct_ingredient
                            },
                        );
                        salsa::tracked_struct::TrackedStructIngredient::new(index, #debug_name_struct)
                    };

                    let field_ingredients = [
                        #(
                            {
                                let index = routes.push(
                                    |jars| {
                                        let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                                        let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar);
                                        &ingredients.#tracked_fields_ingredients[#field_indices]
                                    },
                                    |jars| {
                                        let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars_mut(jars);
                                        let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient_mut(jar);
                                        &mut ingredients.#tracked_fields_ingredients[#field_indices]
                                    },
                                );
                                struct_ingredient.new_field_ingredient(index, #field_indices, #debug_name_fields)
                            },
                        )*
                    ];

                    (struct_ingredient, field_ingredients)
                }
            }
        }
    }

    /// Implementation of `SalsaStructInDb`.
    fn salsa_struct_in_db_impl(&self) -> syn::ItemImpl {
        let ident = self.id_ident();
        let jar_ty = self.jar_ty();
        let tracked_struct_ingredient = self.tracked_struct_ingredient_index();
        parse_quote! {
            impl<DB> salsa::salsa_struct::SalsaStructInDb<DB> for #ident
            where
                DB: ?Sized + salsa::DbWithJar<#jar_ty>,
            {
                fn register_dependent_fn(db: &DB, index: salsa::routes::IngredientIndex) {
                    let (jar, _) = <_ as salsa::storage::HasJar<#jar_ty>>::jar(db);
                    let ingredients = <#jar_ty as salsa::storage::HasIngredientsFor<#ident>>::ingredient(jar);
                    ingredients.#tracked_struct_ingredient.register_dependent_fn(index)
                }
            }
        }
    }

    /// Implementation of `TrackedStructInDb`.
    fn tracked_struct_in_db_impl(&self) -> syn::ItemImpl {
        let ident = self.id_ident();
        let jar_ty = self.jar_ty();
        let tracked_struct_ingredient = self.tracked_struct_ingredient_index();
        parse_quote! {
            impl<DB> salsa::tracked_struct::TrackedStructInDb<DB> for #ident
            where
                DB: ?Sized + salsa::DbWithJar<#jar_ty>,
            {
                fn database_key_index(self, db: &DB) -> salsa::DatabaseKeyIndex {
                    let (jar, _) = <_ as salsa::storage::HasJar<#jar_ty>>::jar(db);
                    let ingredients = <#jar_ty as salsa::storage::HasIngredientsFor<#ident>>::ingredient(jar);
                    ingredients.#tracked_struct_ingredient.database_key_index(self)
                }
            }
        }
    }

    /// The index of the tracked struct ingredient in the ingredient tuple.
    fn tracked_struct_ingredient_index(&self) -> Literal {
        Literal::usize_unsuffixed(0)
    }

    /// The index of the tracked field ingredients array in the ingredient tuple.
    fn tracked_field_ingredients_index(&self) -> Literal {
        Literal::usize_unsuffixed(1)
    }

    /// For this struct, we create a tuple that contains the function ingredients
    /// for each field and the tracked-struct ingredient. These are the indices
    /// of the function ingredients within that tuple.
    fn all_field_indices(&self) -> Vec<Literal> {
        (0..self.all_fields().count())
            .map(Literal::usize_unsuffixed)
            .collect()
    }

    /// For this struct, we create a tuple that contains the function ingredients
    /// for each "other" field and the tracked-struct ingredient. These are the indices
    /// of the function ingredients within that tuple.
    fn all_field_count(&self) -> Literal {
        Literal::usize_unsuffixed(self.all_fields().count())
    }

    /// Indices of each of the id fields
    fn id_field_indices(&self) -> Vec<Literal> {
        self.all_fields()
            .zip(0..)
            .filter(|(field, _)| field.is_id_field())
            .map(|(_, index)| Literal::usize_unsuffixed(index))
            .collect()
    }
}

impl SalsaField {
    /// true if this is an id field
    fn is_id_field(&self) -> bool {
        self.has_id_attr
    }
}
