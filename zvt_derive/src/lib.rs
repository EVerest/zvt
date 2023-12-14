use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Data, Fields};

#[derive(Default)]
struct ZvtBmp {
    number: Option<u16>,
    length_type: proc_macro2::TokenStream,
    encoding_type: proc_macro2::TokenStream,
}

impl Parse for ZvtBmp {
    fn parse(s: ParseStream) -> syn::Result<Self> {
        let mut number = None;
        let mut length_type = quote! {zvt_builder::length::Empty };
        let mut encoding_type = quote! {zvt_builder::encoding::Default};
        loop {
            let ident: syn::Ident = s.parse()?;
            match &ident.to_string() as &str {
                "number" => {
                    let _: syn::Token![=] = s.parse()?;
                    let value: syn::LitInt = s.parse()?;
                    number = Some(value.base10_parse::<u16>()?);
                }
                "length" => {
                    let _: syn::Token![=] = s.parse()?;
                    let e: syn::TypePath = s.parse()?;
                    length_type = quote! {#e};
                }
                "encoding" => {
                    let _: syn::Token![=] = s.parse()?;
                    let e: syn::TypePath = s.parse()?;
                    encoding_type = quote! {#e};
                }
                other => {
                    return Err(s.error(format!("Unexpected identifier: {other}")));
                }
            }
            if s.parse::<syn::Token![,]>().is_err() {
                break;
            }
        }
        Ok(ZvtBmp {
            number,
            length_type,
            encoding_type,
        })
    }
}

#[derive(Default)]
struct ZvtTlv {
    tag: Option<u16>,
    encoding_type: proc_macro2::TokenStream,
}

impl Parse for ZvtTlv {
    fn parse(s: ParseStream) -> syn::Result<Self> {
        let mut tag = None;
        let mut encoding_type = quote! {zvt_builder::encoding::Default};
        loop {
            let ident: syn::Ident = s.parse()?;
            match &ident.to_string() as &str {
                "tag" => {
                    let _: syn::Token![=] = s.parse()?;
                    let value: syn::LitInt = s.parse()?;
                    tag = Some(value.base10_parse::<u16>()?);
                }
                "encoding" => {
                    let _: syn::Token![=] = s.parse()?;
                    let e: syn::TypePath = s.parse()?;
                    encoding_type = quote! {#e};
                }
                other => {
                    return Err(s.error(format!("Unexpected identifier: {other}")));
                }
            }
            if s.is_empty() {
                break;
            }
            s.parse::<syn::Token![,]>()?;
        }
        Ok(ZvtTlv { tag, encoding_type })
    }
}

struct ZvtControlField {
    class: u8,
    instr: u8,
}

impl Parse for ZvtControlField {
    fn parse(s: ParseStream) -> syn::Result<Self> {
        let mut class = None;
        let mut instr = None;
        loop {
            let ident: syn::Ident = s.parse()?;
            match &ident.to_string() as &str {
                "class" => {
                    if class.is_some() {
                        return Err(s.error("Duplicated `class` identifier"));
                    }
                    let _: syn::Token![=] = s.parse()?;
                    let value: syn::LitInt = s.parse()?;
                    class = Some(value.base10_parse::<u8>()?);
                }
                "instr" => {
                    if instr.is_some() {
                        return Err(s.error("Duplicated `instr` identifier"));
                    }
                    let _: syn::Token![=] = s.parse()?;
                    let value: syn::LitInt = s.parse()?;
                    instr = Some(value.base10_parse::<u8>()?);
                }
                other => {
                    return Err(s.error(format!("Unexpected identifier: {other}")));
                }
            }
            if s.is_empty() {
                break;
            }
            s.parse::<syn::Token![,]>()?;
        }
        let class = class.ok_or(s.error("Missing `class` identifier"))?;
        let instr = instr.ok_or(s.error("Missing `instr` identifier"))?;
        Ok(Self { class, instr })
    }
}

/// Optional fields have to be de-serialized separate from the others.
fn is_optional(ty: &syn::Type) -> bool {
    if let syn::Type::Path(ref type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            // Data in a container (e.x. Vec) may be omitted. In this case
            // the container remains empty.
            if segment.ident == "Option" || segment.ident == "Vec" {
                return true;
            }
        }
    }
    false
}

/// Serializes one field of a struct.
fn derive_serialize_field(field: &syn::Field, options: &ZvtBmp) -> proc_macro2::TokenStream {
    let name = field.ident.as_ref().unwrap();
    let ty = &field.ty;
    let ZvtBmp {
        number,
        length_type,
        encoding_type,
    } = options;

    let number_quote = match number {
        None => quote! {None},
        Some(number) => quote! {Some(zvt_builder::Tag(#number))},
    };

    quote! {
        // The `output` and `self.#name` must be defined outside of this macro.
        output.append(&mut <#ty as zvt_builder::ZvtSerializerImpl<#length_type, #encoding_type >>::serialize_tagged(&input.#name, #number_quote));
    }
}

/// Deserializes one untagged field of a struct.
///
/// All un-tagged fields are assumed to be defined before the first tagged field.
/// The order of the un-tagged fields must be the same as they are defined in
/// the struct.
///
/// The generated macro is unhygienic and shall be used inside
/// [derive_deserialize].
fn derive_deserialize_field(field: &syn::Field, options: &ZvtBmp) -> proc_macro2::TokenStream {
    let name = field.ident.as_ref().unwrap();
    let ty = &field.ty;
    let ZvtBmp {
        number: _,
        length_type,
        encoding_type,
    } = options;

    quote! {
        // The `#name` and `bytes` must be defined outside of this macro.
        let (#name, mut bytes) = <#ty as zvt_builder::ZvtSerializerImpl<#length_type, #encoding_type>>::deserialize_tagged(&bytes, None)?;
    }
}

/// Deserializes one tagged field of a struct.
///
/// The tagged fields must come after the un-tagged fields. The order of the
/// tagged fields can to be arbitrary since we can identify the fields by
/// their tag.
///
/// The generated macro is unhygienic and shall be used inside
/// [derive_deserialize].
fn derive_deserialize_field_tagged(
    field: &syn::Field,
    options: &ZvtBmp,
) -> proc_macro2::TokenStream {
    let name = field.ident.as_ref().unwrap();
    let ty = &field.ty;
    let ZvtBmp {
        number,
        length_type,
        encoding_type,
    } = options;

    quote! {
        // The `#name` and `bytes` must be defined outside of this macro. This
        // implements a match arm.
        #number => {
            if ! actual_tags.insert(#number) {
                // The data contains duplicate fields.
                return Err(zvt_builder::ZVTError::DuplicateTag(zvt_builder::Tag(#number)));
            }
            // Remove the number from the required tags.
            required_tags.remove(&#number);

            (#name, bytes) = <#ty as zvt_builder::ZvtSerializerImpl<#length_type, #encoding_type>>::deserialize_tagged(&bytes, Some(zvt_builder::Tag(#number)))?;
        }
    }
}

fn derive_deserialize(
    fields: &syn::FieldsNamed,
    field_options: &[ZvtBmp],
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    assert_eq!(fields.named.len(), field_options.len());

    // Split the fields in positional and optional.
    let mut field_names = Vec::new();
    let mut opt_field_names = Vec::new();
    let mut opt_field_quotes = Vec::new();
    let mut opt_field_tys = Vec::new();
    let mut pos_field_quotes = Vec::new();
    let mut pos_tagged_field_names = Vec::new();
    for (f, opt) in fields.named.iter().zip(field_options) {
        field_names.push(f.ident.as_ref().unwrap());
        match opt.number {
            None => pos_field_quotes.push(derive_deserialize_field(f, opt)),
            Some(number) => {
                opt_field_quotes.push(derive_deserialize_field_tagged(f, opt));
                opt_field_names.push(f.ident.as_ref().unwrap());
                if !is_optional(&f.ty) {
                    pos_tagged_field_names.push(quote! {#number});
                }
                opt_field_tys.push(&f.ty);
            }
        }
    }

    quote! {
        fn decode<'a>(mut bytes: &'a [u8]) -> zvt_builder::ZVTResult<(#name, &'a [u8])> {
            // The untagged fields are the positional fields and we deserialize
            // them first.
            #(#pos_field_quotes;)*

            // We have to track two things: if we have seen a value before and
            // if we have - in the end - seen all required tags (tags of fields
            // which aren't Option<T> type).
            let v = [#(#pos_tagged_field_names,)*];
            let mut required_tags = std::collections::HashSet::<u16>::from(v);
            let mut actual_tags = std::collections::HashSet::<u16>::new();

            let mut curr_len = bytes.len() + 1;
            #(let mut #opt_field_names = <#opt_field_tys>::default();)*
            while ! bytes.is_empty() && curr_len != bytes.len() {
                // Make sure to terminate if we don't make progress.
                curr_len = bytes.len();

                // Try to get the next tag.
                let tag: zvt_builder::Tag = match zvt_builder::encoding::Default::decode(&bytes) {
                    Err(_) => break,
                    Ok(data) => data.0,
                };
                log::debug!("Found tag: 0x{:X}", tag.0);

                // Try to match our tags
                match tag.0 {
                    #(#opt_field_quotes)*
                    _ => {
                        log::debug!("Unhandled tag: 0x{:X}. We try to skip it, but expect trouble.", tag.0);
                        break;
                    }
                }

            }

            // We haven't found all required tags.
            if !required_tags.is_empty() {
                let mut as_vec: Vec<_> = required_tags.into_iter().map(|c| zvt_builder::Tag(c)).collect();
                as_vec.sort_by_key(|t| t.0);
                return Err(zvt_builder::ZVTError::MissingRequiredTags(as_vec));
            }

            Ok((#name{
                #(#field_names: #field_names),*
            }, &bytes))
        }
    }
}

fn derive_serialize(
    fields: &syn::FieldsNamed,
    field_options: &[ZvtBmp],
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let field_tokens = fields
        .named
        .iter()
        .zip(field_options.iter())
        .map(|(name, option)| derive_serialize_field(name, option));

    quote! {
        fn encode(input: &#name) -> Vec<u8> {
            let mut output = Vec::new();
            #(#field_tokens)*
            output
        }
    }
}

fn derive_zvt_command_trait(
    ast: &syn::DeriveInput,
    struct_options: &Option<ZvtControlField>,
) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    match struct_options {
        None => quote! {
            impl zvt_builder::ZvtSerializer for # name {}
        },
        Some(opts) => {
            let generics = &ast.generics;
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            let class = opts.class;
            let instr = opts.instr;
            quote! {
                impl #impl_generics zvt_builder::ZvtCommand for #name #ty_generics #where_clause{
                    const CLASS: u8 = #class;
                    const INSTR: u8 = #instr;
                }
            }
        }
    }
}

fn derive(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    // TODO(hrapp): These should return compile_error! instead of panic.
    // Check the input
    let Data::Struct(ref s) = ast.data else {
        panic!("Only structs are supported");
    };

    let Fields::Named(ref fields) = s.fields else {
        panic!("Only named structs are supported");
    };

    let name = &ast.ident;

    // Get the struct-options (the outer attributes).
    let mut struct_options = None;
    for attr in &ast.attrs {
        if attr.path().is_ident("zvt_control_field") {
            if struct_options.is_some() {
                panic!("Duplicated `zvt_control_field` tag.")
            }
            let syn::Meta::List(meta) = &attr.meta else {
                panic!("We only support List attributes");
            };
            struct_options =
                Some(syn::parse::<ZvtControlField>(meta.tokens.clone().into()).unwrap());
        }
    }

    // Get the field options (the inner attributes).
    let mut field_options = Vec::new();
    for f in &fields.named {
        let options = match f.attrs.len() {
            0 => ZvtBmp {
                number: None,
                length_type: quote! {zvt_builder::length::Empty},
                encoding_type: quote! {zvt_builder::encoding::Default},
            },
            1 => {
                let attr = &f.attrs[0];
                let name = attr.path().get_ident().unwrap().to_string();
                let syn::Meta::List(meta) = &attr.meta else {
                    panic!("We only support List attributes");
                };
                match name.as_str() {
                    "zvt_bmp" => syn::parse(meta.tokens.clone().into()).unwrap(),
                    "zvt_tlv" => {
                        let tlv = syn::parse::<ZvtTlv>(meta.tokens.clone().into()).unwrap();
                        ZvtBmp {
                            number: tlv.tag,
                            length_type: quote! {zvt_builder::length::Tlv},
                            encoding_type: tlv.encoding_type,
                        }
                    }
                    _ => panic!("Unsupported tag {}", name),
                }
            }
            _ => panic!("Zvt supports only one attribute."),
        };
        field_options.push(options);
    }

    let zvt_serialize = derive_serialize(fields, &field_options, name);
    let zvt_deserialize = derive_deserialize(fields, &field_options, name);
    let zvt_command = derive_zvt_command_trait(ast, &struct_options);

    let gen = quote! {
        impl zvt_builder::encoding::Encoding<#name> for zvt_builder::encoding::Default{
            #zvt_serialize
            #zvt_deserialize
        }

        impl <L: zvt_builder::length::Length, TE: zvt_builder::encoding::Encoding<zvt_builder::Tag>> zvt_builder::ZvtSerializerImpl<L, zvt_builder::encoding::Default, TE> for #name {}

        #zvt_command
    };
    gen.into()
}

#[proc_macro_derive(Zvt, attributes(zvt_bmp, zvt_tlv, zvt_control_field))]
pub fn parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    // Build the trait implementation
    derive(&ast)
}

#[proc_macro_derive(ZvtEnum)]
pub fn zvt_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let Data::Enum(ref s) = ast.data else {
        panic!("Only enums are supported - it's in the name");
    };
    let mut variants = Vec::new();
    for variant in &s.variants {
        let Fields::Unnamed(field) = &variant.fields else {
            panic!("We need unnamed fields");
        };
        if field.unnamed.len() != 1 {
            panic!("We need only one element");
        }
        let name = &variant.ident;
        let ty = &field.unnamed[0].ty;
        variants.push(quote!{
            (<#ty as zvt_builder::ZvtCommand>::CLASS, <#ty as zvt_builder::ZvtCommand>::INSTR) => {
                return Ok(Self::#name(<#ty as zvt_builder::ZvtSerializer>::zvt_deserialize(&bytes)?.0));
            }
        });
    }
    let name = ast.ident;
    quote! {
        impl zvt_builder::ZvtParser for #name {
            fn zvt_parse(bytes: &[u8]) -> zvt_builder::ZVTResult<Self> {
                if bytes.len() < 2 {
                    return Err(zvt_builder::ZVTError::IncompleteData);
                }
                match (bytes[0], bytes[1]) {
                    #(#variants,)*
                    _ => return Err(zvt_builder::ZVTError::WrongTag(zvt_builder::Tag(0)))
                }
            }
        }
    }
    .into()
}
