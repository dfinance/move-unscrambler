extern crate handlebars;
extern crate handlebars_misc_helpers;

use std::{collections::HashMap, sync::Arc};
use anyhow::{bail, format_err, Result};
use serde::Serialize;
use handlebars::*;
use crate::cli::{OUTPUT_STDOUT, Output, OutputFmt};

type FilesMap = HashMap<String, String>;

const MAIN_OUTPUT_FILENAME: &str = "output";
const REPORT_TEMPLATE_NAME: &str = "document";

mod defaults {
    pub static REPORT_TEMPLATE_MD_SRC: &str = include_str!("templates/doc.hbs.md");
}

pub fn render<Ctx: Serialize>(cfg: &Output, ctx: Ctx) -> Result<()> {
    if !cfg!(target_arch = "wasm32") {
        prepare_fs(&cfg)?;

        let _ctx = {
            use serde_json::json;
            json!({
                "root": {
                    "address": "0xROOT::Mod",
                    "is_script": true,
                    "entry_points": [
                        {
                            "address": "0xROOT::Mod::Fn0",
                        },
                        {
                            "address": "0xROOT::Mod::Fn1",
                        },
                        ]
                },
                "dependencies":{
                    "functions": [
                        {
                            "address": "0xDEP::Foo::Fn0",
                        },
                        {
                            "address": "0xDEP::Foo::Fn1",
                        },
                    ],

                    "structs": [
                        {
                            "address": "0xDEP::Foo::Struct0",
                        },
                        {
                            "address": "0xDEP::Foo::Struct1",
                        },
                    ]
                },
            })
        };

        let output = render_fmt(cfg, &ctx)?;

        // TODO: save used diagrams to cfg.dir
        // TODO: save `output` to cfg.dir
        error!("OUTPUT:");
        for (k, v) in output {
            error!("{}: {}", k, v);
        }
    } else {
        unimplemented!("not yet");
    }

    Ok(())
}

fn render_json<Ctx: Serialize>(_cfg: &Output, ctx: Ctx) -> Result<String> {
    serde_json::to_string_pretty(&ctx).map_err(anyhow::Error::msg)
}

fn render_fmt<Ctx: Serialize>(cfg: &Output, ctx: Ctx) -> Result<FilesMap> {
    let mut files: HashMap<String, String> = Default::default();

    // if simple serialize requested
    match &cfg.format {
        OutputFmt::Json => {
            files.insert(
                format!("{}.json", MAIN_OUTPUT_FILENAME),
                render_json(cfg, &ctx)?,
            );
            return Ok(files);
        }
        OutputFmt::Yaml => unimplemented!("not yet"),
        _ => {}
    }

    let hb = setup_tmt(cfg)?;

    {
        // TODO: add rendered diagrams into the ctx
    }

    let fileext = match &cfg.format {
        OutputFmt::Markdown => "md",
        OutputFmt::Html => unimplemented!("not yet"),
        _ => unreachable!(),
    };

    hb.render(REPORT_TEMPLATE_NAME, &ctx)
        .map(|output| {
            let filename = format!("{}.{}", MAIN_OUTPUT_FILENAME, fileext);
            files.insert(filename, output);
            files
        })
        .map_err(anyhow::Error::msg)
}

fn setup_tmt<'hb>(cfg: &Output) -> Result<Handlebars<'hb>> {
    let mut hb = Handlebars::new();

    hb.set_strict_mode(true);
    hb.source_map_enabled(true);

    {
        handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));
        hb.register_helper("hex", Box::new(hex));

        handlebars_misc_helpers::setup_handlebars(&mut hb);
    }

    match &cfg.format {
        OutputFmt::Markdown => {
            // TODO: register all defaults:
            hb.register_template_string(REPORT_TEMPLATE_NAME, defaults::REPORT_TEMPLATE_MD_SRC)?;
            // hb.register_partial("function", "FN: {{fn}}\nTHIS: {{this}}")?;
            // TODO: register all templates in the user's templates directory:..
        }
        OutputFmt::Html => unimplemented!("not yet"),
        _ => {}
    }

    Ok(hb)
}

fn prepare_fs(cfg: &Output) -> Result<()> {
    if cfg.dir.as_os_str().to_str() != Some(OUTPUT_STDOUT) {
        let out_dir = cfg.dir.as_path();
        std::fs::create_dir_all(out_dir).map_err(anyhow::Error::msg)
    } else {
        Ok(())
    }
}
