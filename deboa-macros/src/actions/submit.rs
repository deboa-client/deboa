use syn::{
    Expr, Ident, Result, Token,
    parse::{Parse, ParseStream},
};

pub(crate) struct SubmitArgs {
    pub(crate) url: Option<Expr>,
    pub(crate) headers: Option<Expr>,
    pub(crate) client: Option<Expr>,
    pub(crate) method: Option<Expr>,
    pub(crate) data: Option<Expr>,
}

impl Parse for SubmitArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut url = None;
        let mut headers = None;
        let mut client = None;
        let mut method = None;
        let mut data = None;

        // Loop through the comma-separated key:value pairs
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=>]>()?;

            // Check which key we found and look for duplicates
            match key
                .to_string()
                .as_str()
            {
                "url" => {
                    if url.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'url' parameter"));
                    }
                    url = Some(input.parse()?);
                }
                "headers" => {
                    if headers.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'headers' parameter"));
                    }
                    headers = Some(input.parse()?);
                }
                "client" => {
                    if client.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'client' parameter"));
                    }
                    client = Some(input.parse()?);
                }
                "method" => {
                    if method.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'method' parameter"));
                    }
                    method = Some(input.parse()?);
                }
                "data" => {
                    if data.is_some() {
                        return Err(syn::Error::new_spanned(&key, "Duplicate 'data' parameter"));
                    }
                    data = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            // Check if there's another parameter
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(SubmitArgs { url, headers, client, method, data })
    }
}
