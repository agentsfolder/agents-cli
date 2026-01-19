use std::collections::BTreeMap;

use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

pub fn register_helpers(hb: &mut Handlebars<'_>) {
    hb.register_helper("indent", Box::new(IndentHelper));
    hb.register_helper("join", Box::new(JoinHelper));
    hb.register_helper("toJson", Box::new(ToJsonHelper { jsonc: false }));
    hb.register_helper("toJsonc", Box::new(ToJsonHelper { jsonc: true }));
    hb.register_helper("toYaml", Box::new(ToYamlHelper));
    hb.register_helper("frontmatter", Box::new(FrontmatterHelper));
    hb.register_helper("generatedStamp", Box::new(GeneratedStampHelper));
}

struct IndentHelper;

impl HelperDef for IndentHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let n = h
            .param(1)
            .and_then(|v| v.value().as_i64())
            .unwrap_or(0)
            .max(0) as usize;

        let pad = " ".repeat(n);
        let mut first = true;
        for line in text.split('\n') {
            if !first {
                out.write("\n")?;
            }
            first = false;
            if !line.is_empty() {
                out.write(&pad)?;
            }
            out.write(line)?;
        }

        Ok(())
    }
}

struct JoinHelper;

impl HelperDef for JoinHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let sep = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");

        let list = h.param(0).map(|v| v.value()).cloned();
        if let Some(val) = list {
            if let Some(arr) = val.as_array() {
                let mut first = true;
                for item in arr {
                    if !first {
                        out.write(sep)?;
                    }
                    first = false;
                    if let Some(s) = item.as_str() {
                        out.write(s)?;
                    } else {
                        out.write(&item.to_string())?;
                    }
                }
            }
        }

        Ok(())
    }
}

struct ToJsonHelper {
    jsonc: bool,
}

impl HelperDef for ToJsonHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let v = h.param(0).map(|p| p.value()).cloned().unwrap_or_default();
        let normalized = normalize_json_value(&v);

        // Pretty JSON for readability; stable ordering via BTreeMap.
        let mut s = serde_json::to_string_pretty(&normalized).unwrap_or_else(|_| "{}".to_string());
        if self.jsonc {
            // JSONC is identical for now; stamping is handled elsewhere.
        }
        // Ensure trailing newline behavior is controlled by caller.
        if s.ends_with('\n') {
            s.pop();
        }
        out.write(&s)?;
        Ok(())
    }
}

struct ToYamlHelper;

impl HelperDef for ToYamlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let v = h.param(0).map(|p| p.value()).cloned().unwrap_or_default();
        let normalized = normalize_json_value(&v);

        // Convert via serde_yaml; ordering preserved via BTreeMap normalization.
        let yaml = serde_yaml::to_string(&normalized).unwrap_or_default();
        out.write(yaml.trim_end_matches('\n'))?;
        Ok(())
    }
}

struct FrontmatterHelper;

impl HelperDef for FrontmatterHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let v = h.param(0).map(|p| p.value()).cloned().unwrap_or_default();
        let normalized = normalize_json_value(&v);
        let yaml = serde_yaml::to_string(&normalized).unwrap_or_default();

        out.write("---\n")?;
        out.write(yaml.trim_end_matches('\n'))?;
        out.write("\n---\n")?;
        Ok(())
    }
}

struct GeneratedStampHelper;

impl HelperDef for GeneratedStampHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let v = h.param(0).map(|p| p.value()).cloned().unwrap_or_default();
        let normalized = normalize_json_value(&v);

        let json = serde_json::to_string(&normalized).unwrap_or_else(|_| "{}".to_string());
        out.write("<!-- @generated by agents: ")?;
        out.write(&json)?;
        out.write(" -->")?;
        Ok(())
    }
}

fn normalize_json_value(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut b: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            for (k, vv) in map {
                b.insert(k.clone(), normalize_json_value(vv));
            }
            serde_json::to_value(b).unwrap_or_else(|_| serde_json::Value::Object(map.clone()))
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(normalize_json_value).collect())
        }
        _ => v.clone(),
    }
}
