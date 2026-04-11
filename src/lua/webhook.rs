use mlua::prelude::*;
use serde_json::{json, Map, Value};
use std::time::Duration;

// ── LuaEmbed ──────────────────────────────────────────────────────────────────

pub(super) struct LuaEmbed {
    pub enabled:     bool,
    pub color:       u32,
    pub title:       String,
    pub kind:        String,  // exposed as "type"
    pub description: String,
    pub url:         String,
    pub thumbnail:   String,
    pub image:       String,
    pub fields:      Vec<(String, String, bool)>,  // (name, value, inline)
    pub footer_key:  LuaRegistryKey,  // table { text, icon_url }
    pub author_key:  LuaRegistryKey,  // table { name, url, icon_url }
}

impl LuaUserData for LuaEmbed {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("use", |_, e| Ok(e.enabled));
        fields.add_field_method_set("use", |_, e, v: bool| { e.enabled = v; Ok(()) });

        fields.add_field_method_get("color", |_, e| Ok(e.color));
        fields.add_field_method_set("color", |_, e, v: u32| { e.color = v; Ok(()) });

        fields.add_field_method_get("title", |_, e| Ok(e.title.clone()));
        fields.add_field_method_set("title", |_, e, v: String| { e.title = v; Ok(()) });

        fields.add_field_method_get("type", |_, e| Ok(e.kind.clone()));
        fields.add_field_method_set("type", |_, e, v: String| { e.kind = v; Ok(()) });

        fields.add_field_method_get("description", |_, e| Ok(e.description.clone()));
        fields.add_field_method_set("description", |_, e, v: String| { e.description = v; Ok(()) });

        fields.add_field_method_get("url", |_, e| Ok(e.url.clone()));
        fields.add_field_method_set("url", |_, e, v: String| { e.url = v; Ok(()) });

        fields.add_field_method_get("thumbnail", |_, e| Ok(e.thumbnail.clone()));
        fields.add_field_method_set("thumbnail", |_, e, v: String| { e.thumbnail = v; Ok(()) });

        fields.add_field_method_get("image", |_, e| Ok(e.image.clone()));
        fields.add_field_method_set("image", |_, e, v: String| { e.image = v; Ok(()) });

        // Returns the live table — mutations like embed.footer.text = "..." persist.
        fields.add_field_method_get("footer", |lua, e| {
            lua.registry_value::<LuaTable>(&e.footer_key)
        });
        fields.add_field_method_get("author", |lua, e| {
            lua.registry_value::<LuaTable>(&e.author_key)
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addField", |_, e, (name, value, inline): (String, String, bool)| {
            e.fields.push((name, value, inline));
            Ok(())
        });
    }
}

// ── LuaWebhook ────────────────────────────────────────────────────────────────

pub(super) struct LuaWebhook {
    pub url:        String,
    pub content:    String,
    pub username:   String,
    pub avatar_url: String,
    pub embed1_key: LuaRegistryKey,  // LuaEmbed userdata
    pub embed2_key: LuaRegistryKey,  // LuaEmbed userdata
}

impl LuaUserData for LuaWebhook {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("url", |_, w| Ok(w.url.clone()));
        fields.add_field_method_set("url", |_, w, v: String| { w.url = v; Ok(()) });

        fields.add_field_method_get("content", |_, w| Ok(w.content.clone()));
        fields.add_field_method_set("content", |_, w, v: String| { w.content = v; Ok(()) });

        fields.add_field_method_get("username", |_, w| Ok(w.username.clone()));
        fields.add_field_method_set("username", |_, w, v: String| { w.username = v; Ok(()) });

        fields.add_field_method_get("avatar_url", |_, w| Ok(w.avatar_url.clone()));
        fields.add_field_method_set("avatar_url", |_, w, v: String| { w.avatar_url = v; Ok(()) });

        // Returns the live LuaEmbed userdata — embed1.title = "..." persists.
        fields.add_field_method_get("embed1", |lua, w| {
            lua.registry_value::<LuaAnyUserData>(&w.embed1_key)
        });
        fields.add_field_method_get("embed2", |lua, w| {
            lua.registry_value::<LuaAnyUserData>(&w.embed2_key)
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("makeContent", |lua, w, ()| {
            build_payload(lua, w).map(|v| v.to_string())
        });

        methods.add_method("send", |lua, w, ()| {
            let payload = build_payload(lua, w)?.to_string();
            let _ = post_json(&w.url, &payload, None);
            Ok(())
        });

        // edit(message_id) — PATCH {url}/messages/{message_id}
        methods.add_method("edit", |lua, w, message_id: u64| {
            let payload = build_payload(lua, w)?.to_string();
            let edit_url = format!("{}/messages/{}", w.url.trim_end_matches('/'), message_id);
            let _ = patch_json(&edit_url, &payload);
            Ok(())
        });
    }
}

// ── JSON helpers ──────────────────────────────────────────────────────────────

fn embed_to_value(embed: &LuaEmbed, lua: &Lua) -> LuaResult<Option<Value>> {
    if !embed.enabled {
        return Ok(None);
    }

    let mut obj = Map::new();

    if !embed.title.is_empty()       { obj.insert("title".into(),       json!(embed.title)); }
    if !embed.description.is_empty() { obj.insert("description".into(), json!(embed.description)); }
    if !embed.url.is_empty()         { obj.insert("url".into(),          json!(embed.url)); }
    if embed.color != 0              { obj.insert("color".into(),        json!(embed.color)); }
    if !embed.image.is_empty()       { obj.insert("image".into(),       json!({ "url": embed.image })); }
    if !embed.thumbnail.is_empty()   { obj.insert("thumbnail".into(),   json!({ "url": embed.thumbnail })); }

    let footer_tbl = lua.registry_value::<LuaTable>(&embed.footer_key)?;
    let footer_text: Option<String> = footer_tbl.get("text")?;
    let footer_icon: Option<String> = footer_tbl.get("icon_url")?;
    if footer_text.is_some() || footer_icon.is_some() {
        let mut f = Map::new();
        if let Some(t) = footer_text { f.insert("text".into(),     json!(t)); }
        if let Some(i) = footer_icon { f.insert("icon_url".into(), json!(i)); }
        obj.insert("footer".into(), Value::Object(f));
    }

    let author_tbl = lua.registry_value::<LuaTable>(&embed.author_key)?;
    let author_name: Option<String> = author_tbl.get("name")?;
    let author_url:  Option<String> = author_tbl.get("url")?;
    let author_icon: Option<String> = author_tbl.get("icon_url")?;
    if author_name.is_some() {
        let mut a = Map::new();
        if let Some(n) = author_name { a.insert("name".into(),     json!(n)); }
        if let Some(u) = author_url  { a.insert("url".into(),      json!(u)); }
        if let Some(i) = author_icon { a.insert("icon_url".into(), json!(i)); }
        obj.insert("author".into(), Value::Object(a));
    }

    if !embed.fields.is_empty() {
        let flds: Vec<Value> = embed.fields.iter()
            .map(|(n, v, i)| json!({ "name": n, "value": v, "inline": i }))
            .collect();
        obj.insert("fields".into(), json!(flds));
    }

    Ok(Some(Value::Object(obj)))
}

fn build_payload(lua: &Lua, w: &LuaWebhook) -> LuaResult<Value> {
    let mut obj = Map::new();

    if !w.content.is_empty()    { obj.insert("content".into(),    json!(w.content)); }
    if !w.username.is_empty()   { obj.insert("username".into(),   json!(w.username)); }
    if !w.avatar_url.is_empty() { obj.insert("avatar_url".into(), json!(w.avatar_url)); }

    let mut embeds: Vec<Value> = Vec::new();

    let ud1 = lua.registry_value::<LuaAnyUserData>(&w.embed1_key)?;
    {
        let e1 = ud1.borrow::<LuaEmbed>()?;
        if let Some(v) = embed_to_value(&e1, lua)? { embeds.push(v); }
    }

    let ud2 = lua.registry_value::<LuaAnyUserData>(&w.embed2_key)?;
    {
        let e2 = ud2.borrow::<LuaEmbed>()?;
        if let Some(v) = embed_to_value(&e2, lua)? { embeds.push(v); }
    }

    if !embeds.is_empty() {
        obj.insert("embeds".into(), json!(embeds));
    }

    Ok(Value::Object(obj))
}

// ── HTTP helpers (ureq) ───────────────────────────────────────────────────────

fn make_agent() -> ureq::Agent {
    ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .http_status_as_error(false)
            .build(),
    )
}

fn post_json(url: &str, body: &str, query: Option<&str>) -> Result<(), String> {
    let full_url = match query {
        Some(q) => format!("{url}?{q}"),
        None    => url.to_string(),
    };
    let req = ureq::http::Request::builder()
        .method("POST")
        .uri(&full_url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| e.to_string())?;
    make_agent().run(req).map_err(|e| e.to_string())?;
    Ok(())
}

fn patch_json(url: &str, body: &str) -> Result<(), String> {
    let req = ureq::http::Request::builder()
        .method("PATCH")
        .uri(url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| e.to_string())?;
    make_agent().run(req).map_err(|e| e.to_string())?;
    Ok(())
}

// ── Registration ──────────────────────────────────────────────────────────────

fn make_embed(lua: &Lua) -> LuaResult<LuaEmbed> {
    Ok(LuaEmbed {
        enabled:     false,
        color:       0,
        title:       String::new(),
        kind:        "rich".to_string(),
        description: String::new(),
        url:         String::new(),
        thumbnail:   String::new(),
        image:       String::new(),
        fields:      Vec::new(),
        footer_key:  lua.create_registry_value(lua.create_table()?)?,
        author_key:  lua.create_registry_value(lua.create_table()?)?,
    })
}

pub(super) fn register_webhook(lua: &Lua) -> LuaResult<()> {
    let webhook_class = lua.create_table()?;

    webhook_class.set("new", lua.create_function(|lua, url: Option<String>| {
        let embed1 = lua.create_userdata(make_embed(lua)?)?;
        let embed2 = lua.create_userdata(make_embed(lua)?)?;

        Ok(LuaWebhook {
            url:        url.unwrap_or_default(),
            content:    String::new(),
            username:   String::new(),
            avatar_url: String::new(),
            embed1_key: lua.create_registry_value(embed1)?,
            embed2_key: lua.create_registry_value(embed2)?,
        })
    })?)?;

    lua.globals().set("Webhook", webhook_class)?;
    Ok(())
}
