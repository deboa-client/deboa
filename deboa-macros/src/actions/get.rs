use syn::{
    Expr, Ident, Result, Token, Type,
    parse::{Parse, ParseStream},
};

pub(crate) struct GetArgs {
    pub(crate) url: Option<Expr>,
    pub(crate) headers: Option<Expr>,
    pub(crate) client: Option<Expr>,
    pub(crate) res_body_ty: Option<Ident>,
    pub(crate) res_ty: Option<Type>,
}

impl Parse for GetArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut url = None;
        let mut headers = None;
        let mut client = None;
        let mut res_body_ty = None;
        let mut res_ty = None;

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
                        return Err(input.error("duplicate key: url"));
                    }
                    let val: Expr = input.parse()?;
                    url = Some(val);
                }
                "headers" => {
                    if headers.is_some() {
                        return Err(input.error("duplicate key: headers"));
                    }
                    let val: Expr = input.parse()?;
                    headers = Some(val);
                }
                "client" => {
                    if client.is_some() {
                        return Err(input.error("duplicate key: client"));
                    }
                    let val: Expr = input.parse()?;
                    client = Some(val);
                }
                "res_body_ty" => {
                    if res_body_ty.is_some() {
                        return Err(input.error("duplicate key: res_body_ty"));
                    }
                    let val: Ident = input.parse()?;
                    res_body_ty = Some(val);
                }
                "res_ty" => {
                    if res_ty.is_some() {
                        return Err(input.error("duplicate key: res_ty"));
                    }
                    let val: Type = input.parse()?;
                    res_ty = Some(val);
                }
                _ => {
                    return Err(input.error(format!("unknown key: {}", key)));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(GetArgs { url, headers, client, res_body_ty, res_ty })
    }
}
