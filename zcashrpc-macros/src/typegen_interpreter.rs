use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
struct TemplateElementsBuilder {
    rpc_name: String,
    args: Option<syn::Item>,
    responses: Option<syn::Item>,
}
impl TemplateElementsBuilder {
    fn build(self) -> TemplateElements {
        TemplateElements {
            rpc_name: self.rpc_name,
            args: self.args,
            responses: self.responses.expect("Unininitialized??"),
        }
    }
    fn update_if_response_or_args(&mut self, element: syn::Item) {
        // Add assertions about shapes of tg interpretations
        let id = unpack_ident_from_element(&element);
        if id.to_string().rfind("Response").is_some() {
            self.responses = Some(element);
        } else if id.to_string().rfind("Arguments").is_some() {
            match element {
                syn::Item::Enum(ref argcontents) => {
                    validate_enum_arguments_shape(argcontents)
                }
                syn::Item::Struct(ref argcontents) => {
                    validate_struct_arguments_shape(argcontents)
                }
                _ => {
                    panic!()
                }
            }
            self.args = Some(element);
        }
    }
}
fn validate_struct_arguments_shape(argcontents: &syn::ItemStruct) {
    use quote::ToTokens;
    if let syn::Fields::Unnamed(fields) = &argcontents.fields {
        let unnamed_fields_len = fields.unnamed.len();
        assert!(
            (unnamed_fields_len > 0) && (unnamed_fields_len < 7),
            "Unnexpected number of arg fields in: {}",
            argcontents.to_token_stream().to_string(),
        );
    } else {
        panic!("Expected a tuple-like struct.");
    }
}
fn validate_enum_arguments_shape(argcontents: &syn::ItemEnum) {
    assert!(argcontents.variants.len() == 2);
    for variant in &argcontents.variants {
        if let syn::Fields::Unnamed(fields) = &variant.fields {
            assert!(fields.unnamed.len() == 1, "Unexpected unnamed field.");
        } else {
            panic!(
                "contained unexpected non-unnamed fields: {:#?}",
                variant.fields
            );
        }
    }
}
struct TemplateElements {
    rpc_name: String,
    args: Option<syn::Item>,
    responses: syn::Item,
}
impl TemplateElements {
    fn interpolate_into_method(&self) -> proc_macro2::TokenStream {
        let rpc_name = Ident::new(&self.rpc_name, Span::call_site());
        let responseid = unpack_ident_from_element(&self.responses);
        let rpc_name_string = rpc_name.to_string();
        let (args_param_fragment, serialize_args_fragment) =
            generate_args_frag(&rpc_name, &self.args);
        quote!(
            fn #rpc_name(
                &mut self,
                #args_param_fragment
            ) -> std::pin::Pin<Box<dyn Future<
                Output = ResponseResult<rpc_types::#rpc_name::#responseid>>>> {
                let args_for_make_request = #serialize_args_fragment;
                self.make_request(#rpc_name_string, args_for_make_request)
            }
        )
    }
    fn new(rpc_name: String, mod_contents: Vec<syn::Item>) -> Self {
        //! Takes a typegen generated rpc definition, extracts elements:
        //!   rpc_name: Note the name is converted to a string, because the
        //!   originating span metadata isn't useful, and is potentially
        //!   problematic.
        //!            
        //!   arguments
        //!   responses
        let mut templatebuilder = TemplateElementsBuilder {
            rpc_name,
            args: None,
            responses: None,
        };
        for rpc_element in mod_contents {
            templatebuilder.update_if_response_or_args(rpc_element);
        }
        templatebuilder.build()
    }
}
fn generate_args_frag(
    rpc_name: &Ident,
    tg_argtype: &Option<syn::Item>,
) -> (Option<TokenStream>, TokenStream) {
    if let Some(some_contents) = tg_argtype {
        let argid = unpack_ident_from_element(&some_contents);
        let mut token_args = quote!(args);
        match some_contents {
            syn::Item::Struct(ref argcontents) => {
                if let syn::Fields::Unnamed(fields) = &argcontents.fields {
                    if fields.unnamed.len() == 1 {
                        token_args = quote!([#token_args]);
                    }
                } else {
                    panic!("Argument struct is not unnamed!");
                }
            }
            syn::Item::Enum(_) => {
                token_args = quote!([#token_args]);
            }
            _ => {
                panic!("Neither Struct nor Enum")
            }
        }
        (
            Some(quote!(args: rpc_types::#rpc_name::#argid)),
            quote!(Self::serialize_into_output_format(#token_args)),
        )
    } else {
        (None, quote!(Vec::new()))
    }
}
fn unpack_ident_from_element(item: &syn::Item) -> &syn::Ident {
    use syn::Item;
    match item {
        Item::Struct(ref x) => &x.ident,
        Item::Enum(ref x) => &x.ident,
        Item::Type(ref x) => &x.ident,
        otherwise => {
            use quote::ToTokens as _;
            panic!(
                "Expected Struct, Enum, or Type, found {}",
                otherwise.to_token_stream().to_string()
            )
        }
    }
}
pub(crate) fn generate_procedurecall_trait_declaration() -> TokenStream {
    let source = extract_response_idents();
    let syntax = syn::parse_file(&source).expect("Unable to parse file");
    let mut caller_method_definitions = TokenStream::new();
    for item in syntax.items {
        if let syn::Item::Mod(module) = item {
            if let Some(c) = module.content {
                let template_elements =
                    TemplateElements::new(module.ident.to_string(), c.1);
                caller_method_definitions
                    .extend(template_elements.interpolate_into_method());
            }
        } else {
            panic!("Non module item in toplevel of typegen output.")
        }
    }
    let unittests_of_rpc_methods = quote!();
    quote!(
    pub trait ProcedureCall {
        fn make_request<R>(
            &mut self,
            method: &'static str,
            args: Vec<serde_json::Value>,
        ) -> std::pin::Pin<Box<dyn Future<Output = ResponseResult<R>>>>
        where
            R: DeserializeOwned;
        fn serialize_into_output_format<T: serde::Serialize>(
            args: T,
        ) -> Vec<serde_json::Value> {
            let x = serde_json::json!(args).as_array().unwrap().clone();
            if x[0].is_null() {
                if x.len() != 1 {
                    panic!("WHAAA?")
                } else {
                    Vec::new()
                }
            } else {
                if x.iter().any(|x| x.is_null()) {
                    panic!("WHAAA? number 2")
                } else {
                    x
                }
            }
        }
        #caller_method_definitions
    }
        #[cfg(test)]
        mod __procedurecall_methodtests {
            struct MockClient{
            }
            use super::*;
            #unittests_of_rpc_methods
        }
    )
}
pub fn extract_response_idents() -> String {
    let pathstr =
        format!("{}/../src/client/rpc_types.rs", env!("CARGO_MANIFEST_DIR"));
    let raw_rs = std::path::Path::new(&pathstr);
    let mut src = String::new();
    let mut file = std::fs::File::open(&raw_rs).expect("Unable to open file");
    use std::io::Read as _;
    file.read_to_string(&mut src).expect("Unable to read file");
    src
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use super::*;
    mod negative {
        //! Tests of scenarios we don't expect in common operation
        use super::*;
        #[test]
        #[should_panic(
            expected = "Expected Struct, Enum, or Type, found union Why { \
         what : & mut String , who : * const [Box < i128 >] , }"
        )]
        fn non_struct_non_enum() {
            let ident_we_dont_care_about =
                Ident::new("stringnotappearinginthisoutput", Span::call_site());
            let invalid_item_type = syn::parse_quote![
                union Why {
                    what: &mut String,
                    who: *const [Box<i128>],
                }
            ];
            generate_args_frag(
                &ident_we_dont_care_about,
                &Some(invalid_item_type),
            );
        }
    }
    fn get_getinfo_response() -> syn::ItemStruct {
        syn::parse_quote!(
            #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
            pub struct GetinfoResponse {
                pub proxy: Option<String>,
                pub balance: rust_decimal::Decimal,
                pub blocks: rust_decimal::Decimal,
                pub connections: rust_decimal::Decimal,
                pub difficulty: rust_decimal::Decimal,
                pub errors: String,
                pub keypoololdest: rust_decimal::Decimal,
                pub keypoolsize: rust_decimal::Decimal,
                pub paytxfee: rust_decimal::Decimal,
                pub protocolversion: rust_decimal::Decimal,
                pub relayfee: rust_decimal::Decimal,
                pub testnet: bool,
                pub timeoffset: rust_decimal::Decimal,
                pub unlocked_until: rust_decimal::Decimal,
                pub version: rust_decimal::Decimal,
                pub walletversion: rust_decimal::Decimal,
            }
        )
    }
    fn make_z_getnewaddress_mod_contents() -> [syn::Item; 2] {
        [
            syn::parse_quote!(
                #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
                pub struct ZGetnewaddressArguments(Option<String>);
            ),
            syn::parse_quote!(
                pub type ZGetnewaddressResponse = String;
            ),
        ]
    }
    fn make_z_mergetoaddress_mod_contents() -> [syn::Item; 2] {
        [
            syn::parse_quote!(
                #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
                pub struct ZMergetoaddressArguments(
                    Vec<String>,
                    String,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    Option<rust_decimal::Decimal>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    Option<rust_decimal::Decimal>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    Option<rust_decimal::Decimal>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    Option<String>,
                );
            ),
            syn::parse_quote!(
                #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
                pub struct ZMergetoaddressResponse {
                    pub merging_notes: rust_decimal::Decimal,
                    pub merging_shielded_value: rust_decimal::Decimal,
                    pub merging_transparent_value: rust_decimal::Decimal,
                    pub merging_u_t_x_os: rust_decimal::Decimal,
                    pub opid: String,
                    pub remaining_notes: rust_decimal::Decimal,
                    pub remaining_shielded_value: rust_decimal::Decimal,
                    pub remaining_transparent_value: rust_decimal::Decimal,
                    pub remaining_u_t_x_os: rust_decimal::Decimal,
                }
            ),
        ]
    }
    mod generate_args_frag {
        use super::*;
        #[test]
        fn z_getnewaddress_some_case() {
            let expected_args_method_param = "args : rpc_types :: z_getnewaddress :: ZGetnewaddressArguments".to_string();
            let expected_serial_af_call =
                "Self :: serialize_into_output_format ([args])".to_string();

            let input_rpc_name_id =
                Ident::new("z_getnewaddress", Span::call_site());
            let input_args = make_z_getnewaddress_mod_contents()[0].clone();
            let (wrapped_observed_args_method_param, observed_serial_af_call) =
                generate_args_frag(&input_rpc_name_id, &Some(input_args));
            let observed_args_method_param =
                wrapped_observed_args_method_param.unwrap().to_string();
            let observed_serial_af_call = observed_serial_af_call.to_string();
            testutils::Comparator {
                expected: expected_args_method_param,
                observed: observed_args_method_param,
            }
            .compare();
            testutils::Comparator {
                expected: expected_serial_af_call,
                observed: observed_serial_af_call,
            }
            .compare();
        }
        #[test]
        fn getinfo_none_case() {
            let expected_serial_af_call = quote::quote!(Vec::new()).to_string();

            let input_rpc_name_id = Ident::new("getinfo", Span::call_site());
            let (observed_args_method_param, observed_serial_af_call) =
                generate_args_frag(&input_rpc_name_id, &None);
            assert!(observed_args_method_param.is_none());
            let observed_serial_af_call = observed_serial_af_call.to_string();
            testutils::Comparator {
                expected: expected_serial_af_call,
                observed: observed_serial_af_call,
            }
            .compare();
        }
        fn get_getaddressdeltasarguments_tokens() -> Option<syn::Item> {
            Some(syn::parse_quote!(
                #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
                pub enum GetaddressdeltasArguments {
                    MultiAddress(Arg1),
                    Address(String),
                }
            ))
        }
        #[test]
        fn get_addressdeltas_enumarg_example() {
            // Compare serialize arguments
            let expected =
                quote!(Self::serialize_into_output_format([args])).to_string();
            let input_args = get_getaddressdeltasarguments_tokens();
            let input_rpc_name_id =
                Ident::new("getaddressdeltas", Span::call_site());
            let (args_param_fragment, serialize_argument) =
                generate_args_frag(&input_rpc_name_id, &input_args);
            let observed = serialize_argument.to_string();
            testutils::Comparator { expected, observed }.compare();

            // Compare args_param_fragment
            let expected = quote!(
                args: rpc_types::getaddressdeltas::GetaddressdeltasArguments
            )
            .to_string();
            let observed = args_param_fragment.unwrap().to_string();
            testutils::Comparator { expected, observed }.compare();
        }
        #[should_panic(expected = "Argument struct is not unnamed!")]
        #[test]
        fn named_field_argument_structure() {
            let input_args = syn::parse_quote!(
                struct BadField {
                    naughty: String,
                }
            );
            let input_rpc_name_id =
                Ident::new("NO_REAL_RPC", Span::call_site());

            generate_args_frag(&input_rpc_name_id, &Some(input_args));
        }
    }
    mod interpolate_into_method {
        use super::*;
        #[test]
        fn getinfo_happy_path() {
            let input_mod_contents =
                vec![syn::Item::Struct(get_getinfo_response())];
            let observed = TemplateElements::new(
                "getinfo".to_string(),
                input_mod_contents,
            )
            .interpolate_into_method()
            .to_string();
            #[rustfmt::skip]
            let expected = quote!(
                fn getinfo(
                    &mut self,
                ) ->  std::pin::Pin<Box< dyn Future <
                    Output = ResponseResult<
                        rpc_types::getinfo::GetinfoResponse
                    >>>>
                {
                    let args_for_make_request = Vec::new();
                    self.make_request("getinfo", args_for_make_request)
                }
            )
            .to_string();
            testutils::Comparator { expected, observed }.compare();
        }
        #[test]
        fn z_getnewaddress() {
            //Create expected
            #[rustfmt::skip]
            let expected = quote!(
                fn z_getnewaddress(
                    &mut self,
                    args: rpc_types::z_getnewaddress::ZGetnewaddressArguments
                ) ->  std::pin::Pin<Box< dyn Future <
                    Output = ResponseResult<
                        rpc_types::z_getnewaddress::ZGetnewaddressResponse
                    >>>>
                {
                    let args_for_make_request = Self::serialize_into_output_format([args]);
                    self.make_request("z_getnewaddress", args_for_make_request)
                }
            )
            .to_string();

            //Make observation
            let input_mod_contents = make_z_getnewaddress_mod_contents();
            let observed = TemplateElements::new(
                "z_getnewaddress".to_string(),
                input_mod_contents.to_vec(),
            )
            .interpolate_into_method()
            .to_string();
            testutils::Comparator { expected, observed }.compare();
        }
        #[test]
        fn z_mergetoaddress() {
            #[rustfmt::skip]
            let expected = quote!(
                fn z_mergetoaddress(
                    &mut self,
                    args: rpc_types::z_mergetoaddress::ZMergetoaddressArguments
                ) ->  std::pin::Pin<Box< dyn Future <
                    Output = ResponseResult<
                        rpc_types::z_mergetoaddress::ZMergetoaddressResponse
                    >>>>
                {
                    let args_for_make_request =
                        Self::serialize_into_output_format(args);
                    self.make_request("z_mergetoaddress", args_for_make_request)
                }
            )
            .to_string();

            //Make observation
            let input_mod_contents = make_z_mergetoaddress_mod_contents();
            let observed = TemplateElements::new(
                "z_mergetoaddress".to_string(),
                input_mod_contents.to_vec(),
            )
            .interpolate_into_method()
            .to_string();
            testutils::Comparator { expected, observed }.compare();
        }
    }
}
