use mlua::prelude::*;
use std::time::Duration;

pub(super) struct LuaHttpClient {
    pub url:          String,
    pub method:       String,
    pub content:      Option<String>,
    pub proxy:        Option<String>,
    pub headers_key:  LuaRegistryKey,
}

pub(super) struct LuaHttpResult {
    pub body:       Vec<u8>,
    pub status:     u16,
    pub error_code: i32,
    pub error_msg:  String,
}

// ── LuaHttpResult ─────────────────────────────────────────────────────────────

impl LuaUserData for LuaHttpResult {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("body",   |lua, r| lua.create_string(&r.body));
        fields.add_field_method_get("status", |_, r| Ok(r.status));
        fields.add_field_method_get("error",  |_, r| Ok(r.error_code));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getError", |_, r, ()| Ok(r.error_msg.clone()));
    }
}

// ── LuaHttpClient ─────────────────────────────────────────────────────────────

impl LuaUserData for LuaHttpClient {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("url", |_, c| Ok(c.url.clone()));
        fields.add_field_method_set("url", |_, c, v: String| { c.url = v; Ok(()) });

        fields.add_field_method_get("method", |_, c| Ok(c.method.clone()));
        fields.add_field_method_set("method", |_, c, v: String| {
            c.method = v.to_uppercase();
            Ok(())
        });

        fields.add_field_method_get("content", |_, c| Ok(c.content.clone().unwrap_or_default()));
        fields.add_field_method_set("content", |_, c, v: String| {
            c.content = if v.is_empty() { None } else { Some(v) };
            Ok(())
        });

        // Returns the live Lua table — mutations (headers["key"] = "val") work directly.
        fields.add_field_method_get("headers", |lua, c| {
            lua.registry_value::<LuaTable>(&c.headers_key)
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("setMethod", |_, c, method: String| {
            c.method = method.to_uppercase();
            Ok(())
        });

        // setProxy(Proxy.socks5, "host:port")
        methods.add_method_mut("setProxy", |_, c, (ptype, data): (LuaValue, String)| {
            let scheme = match &ptype {
                LuaValue::Integer(n) => match n {
                    1 => "http",
                    2 => "socks4",
                    _ => "socks5",
                },
                _ => "socks5",
            };
            // Prepend scheme if not already present
            let url = if data.contains("://") {
                data
            } else {
                format!("{scheme}://{data}")
            };
            c.proxy = Some(url);
            Ok(())
        });

        methods.add_method_mut("removeProxy", |_, c, ()| {
            c.proxy = None;
            Ok(())
        });

        methods.add_method("request", |lua, c, ()| {
            let timeout = Duration::from_millis(10_000);

            let mut builder = ureq::config::Config::builder()
                .timeout_global(Some(timeout))
                .http_status_as_error(false);

            if let Some(proxy_url) = &c.proxy {
                if let Ok(proxy) = ureq::Proxy::new(proxy_url) {
                    builder = builder.proxy(Some(proxy));
                }
            }

            let agent = ureq::Agent::new_with_config(builder.build());

            let mut req_builder = ureq::http::Request::builder()
                .method(c.method.as_str())
                .uri(&c.url);

            let headers_table = lua.registry_value::<LuaTable>(&c.headers_key)?;
            for pair in headers_table.pairs::<String, String>() {
                let (name, value) = pair?;
                req_builder = req_builder.header(&name, &value);
            }

            let run_result = match &c.content {
                Some(body) => {
                    let req = req_builder
                        .body(body.clone())
                        .map_err(|e| LuaError::runtime(e.to_string()))?;
                    agent.run(req)
                }
                None => {
                    let req = req_builder
                        .body(())
                        .map_err(|e| LuaError::runtime(e.to_string()))?;
                    agent.run(req)
                }
            };

            match run_result {
                Ok(mut resp) => {
                    let status = resp.status().as_u16();
                    let body = resp
                        .body_mut()
                        .read_to_vec()
                        .map_err(|e| LuaError::runtime(e.to_string()))?;
                    Ok(LuaHttpResult { body, status, error_code: 0, error_msg: String::new() })
                }
                Err(e) => Ok(LuaHttpResult {
                    body:       vec![],
                    status:     0,
                    error_code: 1,
                    error_msg:  e.to_string(),
                }),
            }
        });
    }
}

// ── Registration ──────────────────────────────────────────────────────────────

pub(super) fn register_http_client(lua: &Lua) -> LuaResult<()> {
    // HttpClient.new()
    let http_client = lua.create_table()?;
    http_client.set("new", lua.create_function(|lua, ()| {
        let headers = lua.create_table()?;
        let headers_key = lua.create_registry_value(headers)?;
        Ok(LuaHttpClient {
            url:         String::new(),
            method:      "GET".to_string(),
            content:     None,
            proxy:       None,
            headers_key,
        })
    })?)?;
    lua.globals().set("HttpClient", http_client)?;

    // Proxy enum
    let proxy_enum = lua.create_table()?;
    proxy_enum.set("http",   1i32)?;
    proxy_enum.set("socks4", 2i32)?;
    proxy_enum.set("socks5", 3i32)?;
    lua.globals().set("Proxy", proxy_enum)?;

    Ok(())
}
