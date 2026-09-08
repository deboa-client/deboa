use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, Result, Token, Type,
};

pub(crate) struct QueryArgs {
    pub(crate) url: Option<Expr>,
    pub(crate) headers: Option<Expr>,
    pub(crate) req_body_ty: Option<Ident>,
    pub(crate) data: Option<Expr>,
    pub(crate) client: Option<Expr>,
    pub(crate) res_body_ty: Option<Ident>,
    pub(crate) res_ty: Option<Type>,
}

impl Parse for QueryArgs {
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
                        return Err(tokens.error("duplicate key: url"));
                    }
                    let val: Expr = tokens.parse()?;
                    url = Some(val);
                }
                "headers" => {
                    if headers.is_some() {
                        return Err(tokens.error("Duplicate 'headers' parameter"));
                    }
                    let val: Expr = tokens.parse()?;
                    headers = Some(val);
                }
                "req_body_ty" => {
                    if req_body_ty.is_some() {
                        return Err(tokens.error("Duplicate 'req_body_ty' parameter"));
                    }
                    let val: Ident = tokens.parse()?;
                    req_body_ty = Some(val);
                }
                "data" => {
                    if data.is_some() {
                        return Err(tokens.error("Duplicate 'data' parameter"));
                    }
                    let val: Expr = tokens.parse()?;
                    data = Some(val);
                }
                "client" => {
                    if client.is_some() {
                        return Err(tokens.error("Duplicate 'client' parameter"));
                    }
                    let val: Expr = tokens.parse()?;
                    client = Some(val);
                }
                "res_body_ty" => {
                    if res_body_ty.is_some() {
                        return Err(tokens.error("Duplicate 'res_body_ty' parameter"));
                    }
                    let val: Ident = tokens.parse()?;
                    res_body_ty = Some(val);
                }
                "res_ty" => {
                    if res_ty.is_some() {
                        return Err(tokens.error("Duplicate 'res_ty' parameter"));
                    }
                    let val: Type = tokens.parse()?;
                    res_ty = Some(val);
                }
                _ => {
                    return Err(tokens.error(format!("Unknown parameter: {}", key)));
                }
            }

            // Check if there's another parameter
            if tokens.peek(Token![,]) {
                tokens.parse::<Token![,]>()?;
            }
        }

        Ok(QueryArgs { url, headers, req_body_ty, data, client, res_body_ty, res_ty })
    }
}
