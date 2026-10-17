use syn::{
    Expr, Ident, Result, Token, Type,
    parse::{Parse, ParseStream},
};

pub(crate) struct PutArgs {
    pub(crate) url: Option<Expr>,
    pub(crate) headers: Option<Expr>,
    pub(crate) req_body_ty: Option<Ident>,
    pub(crate) data: Option<Expr>,
    pub(crate) client: Option<Expr>,
    pub(crate) res_body_ty: Option<Ident>,
    pub(crate) res_ty: Option<Type>,
}

impl Parse for PutArgs {
    fn parse(tokens: ParseStream) -> Result<Self> {
        let mut url = None;
        let mut headers = None;
        let mut req_body_ty = None;
        let mut data = None;
        let mut client = None;
        let mut res_body_ty = None;
        let mut res_ty = None;

        // Loop through the comma-separated key:value pairs
        while !tokens.is_empty() {
            let key: Ident = tokens.parse()?;
            tokens.parse::<Token![=>]>()?;

            // Check which key we found and look for duplicates
            match key
                .to_string()
                .as_str()
            {
                "url" => {
                    if url.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'url' parameter"));
                    }
                    url = Some(tokens.parse()?);
                }
                "headers" => {
                    if headers.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'headers' parameter"));
                    }
                    headers = Some(tokens.parse()?);
                }
                "client" => {
                    if client.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'client' parameter"));
                    }
                    client = Some(tokens.parse()?);
                }
                "req_body_ty" => {
                    if req_body_ty.is_some() {
                        return Err(syn::Error::new_spanned(
                            &key,
                            "Duplicate 'req_body_ty' parameter",
                        ));
                    }
                    req_body_ty = Some(tokens.parse()?);
                }
                "data" => {
                    if data.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'data' parameter"));
                    }
                    data = Some(tokens.parse()?);
                }
                "res_body_ty" => {
                    if res_body_ty.is_some() {
                        return Err(syn::Error::new_spanned(
                            &key,
                            "Duplicate 'res_body_ty' parameter",
                        ));
                    }
                    res_body_ty = Some(tokens.parse()?);
                }
                "res_ty" => {
                    if res_ty.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'res_ty' parameter"));
                    }
                    res_ty = Some(tokens.parse()?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            // Check if there's another parameter
            if tokens.peek(Token![,]) {
                tokens.parse::<Token![,]>()?;
            }
        }

        Ok(PutArgs { url, headers, req_body_ty, data, client, res_body_ty, res_ty })
    }
}
