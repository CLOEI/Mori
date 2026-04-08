use mlua::prelude::*;
use std::time::Duration;

#[derive(Default)]
pub(super) struct LuaHttpOptions {
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: Option<u64>,
}

fn lua_http_options(opts: Option<LuaTable>) -> LuaResult<LuaHttpOptions> {
    let Some(opts) = opts else {
        return Ok(LuaHttpOptions::default());
    };

    let body = match opts.get::<LuaValue>("body")? {
        LuaValue::Nil => None,
        LuaValue::String(s) => Some(s.to_str()?.to_string()),
        LuaValue::Integer(n) => Some(n.to_string()),
        LuaValue::Number(n) => Some(n.to_string()),
        LuaValue::Boolean(v) => Some(v.to_string()),
        _ => {
            return Err(LuaError::runtime(
                "http options.body must be a string, number, or boolean",
            ));
        }
    };

    let timeout_ms = opts.get::<Option<u64>>("timeout_ms")?;
    let mut headers = Vec::new();

    if let Some(header_table) = opts.get::<Option<LuaTable>>("headers")? {
        for pair in header_table.pairs::<String, LuaValue>() {
            let (name, value) = pair?;
            let value = match value {
                LuaValue::String(s) => s.to_str()?.to_string(),
                LuaValue::Integer(n) => n.to_string(),
                LuaValue::Number(n) => n.to_string(),
                LuaValue::Boolean(v) => v.to_string(),
                _ => {
                    return Err(LuaError::runtime(
                        "http options.headers values must be strings, numbers, or booleans",
                    ));
                }
            };
            headers.push((name, value));
        }
    }

    Ok(LuaHttpOptions {
        headers,
        body,
        timeout_ms,
    })
}

pub(super) fn make_http_request<'lua>(
    lua: &'lua Lua,
    method: &str,
    url: String,
    opts: Option<LuaTable>,
) -> LuaResult<LuaTable> {
    let opts = lua_http_options(opts)?;
    let timeout = Duration::from_millis(opts.timeout_ms.unwrap_or(10_000));

    let builder = ureq::config::Config::builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false);
    let config = builder.build();
    let agent = ureq::Agent::new_with_config(config);

    let mut request = ureq::http::Request::builder().method(method).uri(&url);
    for (name, value) in opts.headers {
        request = request.header(&name, &value);
    }

    let mut response = match opts.body {
        Some(body) => {
            let request = request
                .body(body)
                .map_err(|e| LuaError::runtime(format!("http request build failed: {e}")))?;
            agent
                .run(request)
                .map_err(|e| LuaError::runtime(format!("http request failed: {e}")))?
        }
        None => {
            let request = request
                .body(())
                .map_err(|e| LuaError::runtime(format!("http request build failed: {e}")))?;
            agent
                .run(request)
                .map_err(|e| LuaError::runtime(format!("http request failed: {e}")))?
        }
    };

    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("").to_string();

    let header_table = lua.create_table()?;
    for (name, value) in response.headers() {
        header_table.set(
            name.as_str().to_ascii_lowercase(),
            String::from_utf8_lossy(value.as_bytes()).to_string(),
        )?;
    }

    let body = response
        .body_mut()
        .read_to_vec()
        .map_err(|e| LuaError::runtime(format!("failed to read http response body: {e}")))?;

    let result = lua.create_table()?;
    result.set("ok", (200..300).contains(&status))?;
    result.set("status", status)?;
    result.set("status_text", status_text)?;
    result.set("headers", header_table)?;
    result.set("body", lua.create_string(&body)?)?;

    Ok(result)
}
