use proc_macro::{self, TokenStream};
use quote::{quote, format_ident};
use syn::{parse_macro_input, parse_quote, Fields, FieldsNamed, FieldsUnnamed, Item, ItemStruct};

#[proc_macro_attribute]
pub fn model(_: TokenStream, input: TokenStream) -> TokenStream {
    let item: Item = parse_macro_input!(input);
    match item {
        Item::Struct(struct_) => struct_impl(&struct_),
        unsupported => syn::Error::new_spanned(unsupported, "#[model] only supports structs.")
            .into_compile_error()
            .into(),
    }
}

fn struct_impl(struct_: &ItemStruct) -> TokenStream {
    match &struct_.fields {
        Fields::Named(fields_named) => struct_named_fields_impl(struct_, fields_named),
        Fields::Unnamed(fields_unnamed) => struct_unnamed_fields_impl(struct_, fields_unnamed),
        unsupported => syn::Error::new_spanned(unsupported, "#[model] only supports named fields.")
            .into_compile_error()
            .into(),
    }
}

fn struct_named_fields_impl(struct_: &ItemStruct, named_fields: &FieldsNamed) -> TokenStream {
    let ident = &struct_.ident;

    let mut struct_ = struct_.clone();
    let mut named_fields = named_fields.clone();

    let mut new_args = Vec::new();
    let mut new_values = Vec::new();
    let mut dict_items = Vec::new();
    for field in &mut named_fields.named {
        let field_ident = &field.ident.clone().unwrap();
        let field_type = &field.ty;
        let field_ident_str = field_ident.to_string();

        new_args.push(quote! {#field_ident: #field_type});
        new_values.push(quote! {#field_ident});

        dict_items.push(quote! {
            dict.set_item(#field_ident_str, self.#field_ident.to_model_dict(py)?)?;
        });

        field.attrs.push(parse_quote! {#[pyo3(get, set)]});
    }
    struct_.fields = Fields::Named(named_fields);

    let output = quote! {
        #[pyo3::pyclass]
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize, schemars::JsonSchema)]
        #struct_

        #[pyo3::pymethods]
        impl #ident {
            #[new]
            fn new(#(#new_args),*) -> Self {
                return Self { #(#new_values),* };
            }

            fn __richcmp__(&self, other: &Self, op: pyo3::basic::CompareOp) -> pyo3::PyResult<bool> {
                match op {
                    pyo3::basic::CompareOp::Eq => Ok(self == other),
                    pyo3::basic::CompareOp::Ne => Ok(self != other),
                    _ => Err(ordantic::OrdanticError::new_err("not implemented"))
                }
            }

            fn dict(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
                self.to_model_dict(py)
            }

            fn json(&self) -> pyo3::PyResult<String> {
                let json_str = serde_json::to_string(&self)
                    .map_err(|_| ordantic::OrdanticError::new_err("failed to serialize"))?;
                Ok(json_str)
            }

            #[classmethod]
            fn parse_raw(_cls: &pyo3::types::PyType, string: &str) -> pyo3::PyResult<Self> {
                let self_ = serde_json::from_str(string)
                    .map_err(|_| ordantic::OrdanticError::new_err("failed to deserialize"))?;
                Ok(self_)
            }

            #[classmethod]
            fn schema<'py>(_cls: &'py pyo3::types::PyType, py: pyo3::Python<'py>) -> pyo3::PyResult<&'py pyo3::types::PyDict> {
                let schema = schemars::schema_for!(#ident);
                let dict = pyo3::types::PyDict::new(py);

                if let Some(meta_schema) = schema.meta_schema {
                    dict.set_item("$schema", meta_schema)?;
                }

                dict.set_item("title", "ExampleModel")?;
                dict.set_item("type", "object")?;

                for (field, definition) in schema.definitions {
                    //dict.set_item(field, definition)?;
                }

                Ok(dict)
            }

            #[classmethod]
            fn schema_json(_cls: &pyo3::types::PyType) -> pyo3::PyResult<String> {
                let schema = schemars::schema_for!(#ident);
                let json_str = serde_json::to_string(&schema)
                    .map_err(|_| ordantic::OrdanticError::new_err("failed to serialize"))?;
                Ok(json_str)
            }

            #[classmethod]
            fn __get_validators__(_cls: &pyo3::types::PyType) -> pyo3::PyResult<ordantic::ValidatorIterator> {
                let schema_validators = Vec::new();
                Ok(ordantic::ValidatorIterator::new(schema_validators))
            }
        }

        impl ordantic::ToModelDict for #ident {
            fn to_model_dict<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<pyo3::PyObject> {
                let dict = pyo3::types::PyDict::new(py);
                #(#dict_items)*
                Ok(dict.into())
            }
        }
    };
    output.into()
}

fn struct_unnamed_fields_impl(struct_: &ItemStruct, unnamed_fields: &FieldsUnnamed) -> TokenStream {
    let ident = &struct_.ident;

    let mut new_args = Vec::new();
    let mut new_values = Vec::new();
    let mut tuple_items = Vec::new();
    for (field_number, field) in &mut unnamed_fields.unnamed.iter().enumerate() {
        let field_type = &field.ty;

        let constructor_name = format_ident!("val_{}", field_number);
        new_args.push(quote! {#constructor_name: #field_type});
        new_values.push(constructor_name);

        tuple_items.push(quote! { self.#field_number.to_model_dict(py)? });
    }
    let output = quote! {
        #[pyo3::pyclass]
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize, schemars::JsonSchema)]
        #struct_

        #[pyo3::pymethods]
        impl #ident {
            #[new]
            fn new(#(#new_args),*) -> Self {
                return Self ( #(#new_values),* );
            }

            fn __richcmp__(&self, other: &Self, op: pyo3::basic::CompareOp) -> pyo3::PyResult<bool> {
                match op {
                    pyo3::basic::CompareOp::Eq => Ok(self == other),
                    pyo3::basic::CompareOp::Ne => Ok(self != other),
                    _ => Err(ordantic::OrdanticError::new_err("not implemented"))
                }
            }

            fn dict(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
                self.to_model_dict(py)
            }

            fn json(&self) -> pyo3::PyResult<String> {
                let json_str = serde_json::to_string(&self)
                    .map_err(|_| ordantic::OrdanticError::new_err("failed to serialize"))?;
                Ok(json_str)
            }

            #[classmethod]
            fn parse_raw(_cls: &pyo3::types::PyType, string: &str) -> pyo3::PyResult<Self> {
                let self_ = serde_json::from_str(string)
                    .map_err(|_| ordantic::OrdanticError::new_err("failed to deserialize"))?;
                Ok(self_)
            }

            #[classmethod]
            fn schema<'py>(_cls: &'py pyo3::types::PyType, py: pyo3::Python<'py>) -> pyo3::PyResult<&'py pyo3::types::PyDict> {
                let schema = schemars::schema_for!(#ident);
                let dict = pyo3::types::PyDict::new(py);

                if let Some(meta_schema) = schema.meta_schema {
                    dict.set_item("$schema", meta_schema)?;
                }

                dict.set_item("title", "ExampleModel")?;
                dict.set_item("type", "object")?;

                for (field, definition) in schema.definitions {
                    //dict.set_item(field, definition)?;
                }

                Ok(dict)
            }

            #[classmethod]
            fn schema_json(_cls: &pyo3::types::PyType) -> pyo3::PyResult<String> {
                let schema = schemars::schema_for!(#ident);
                let json_str = serde_json::to_string(&schema)
                    .map_err(|_| ordantic::OrdanticError::new_err("failed to serialize"))?;
                Ok(json_str)
            }

            #[classmethod]
            fn __get_validators__(_cls: &pyo3::types::PyType) -> pyo3::PyResult<ordantic::ValidatorIterator> {
                let schema_validators = Vec::new();
                Ok(ordantic::ValidatorIterator::new(schema_validators))
            }
        }

        impl ordantic::ToModelDict for #ident {
            fn to_model_dict<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<pyo3::PyObject> {
                let tuple = pyo3::types::PyTuple::new(py, vec![#(#tuple_items)*]);
                Ok(dict.into())
            }
        }
    };
    output.into()
}
