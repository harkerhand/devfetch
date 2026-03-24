use rsenv::{cli, collect, format, model, presets};
use std::collections::BTreeMap;

use rsenv::model::RunOptions;
use rsenv::presets::PresetSpec;

fn build_props_from_flags(args: &cli::Cli, defaults: &PresetSpec) -> PresetSpec {
    let mut out = BTreeMap::new();

    let selected = [
        ("System", args.categories.system),
        ("Browsers", args.categories.browser),
        ("SDKs", args.categories.sdk),
        ("IDEs", args.categories.ide),
        ("Languages", args.categories.languages),
        ("Managers", args.categories.manager),
        ("Binaries", args.categories.binary),
        ("Servers", args.categories.server),
        ("Virtualization", args.categories.r#virtual),
        ("Utilities", args.categories.util),
        ("Databases", args.categories.database),
    ];

    for (category, enabled) in selected {
        if !enabled {
            continue;
        }
        if let Some(default_items) = defaults.get(category) {
            out.insert(category.to_string(), default_items.clone());
        }
    }

    out
}

fn defaults_without_packages() -> PresetSpec {
    presets::defaults()
        .into_iter()
        .filter(|(_, v)| v.is_some())
        .collect()
}

fn options_from_args(args: &cli::Cli) -> RunOptions {
    RunOptions {
        json: args.output.json,
        markdown: args.output.markdown,
        toml: args.output.toml,
        show_not_found: args.output.show_not_found,
        duplicates: args.output.duplicates,
        full_tree: args.output.full_tree,
    }
}

fn main() {
    let args = cli::parse_args();

    let options = options_from_args(&args);
    let defaults = presets::defaults();
    let props = build_props_from_flags(&args, &defaults);

    let spec: PresetSpec;

    if args.all {
        spec = defaults_without_packages();
    } else if let Some(helper) = args.helper.as_deref() {
        match collect::collect_helper(helper, &options) {
            Some(node) => {
                let mut wrapper = BTreeMap::new();
                wrapper.insert(helper.to_string(), node);
                let rendered = format::render(
                    model::Node::Obj(wrapper),
                    &RunOptions {
                        json: true,
                        ..options.clone()
                    },
                );
                println!("{rendered}");
                return;
            }
            None => {
                eprintln!("未找到");
                std::process::exit(2);
            }
        }
    } else if props.is_empty() {
        spec = defaults_without_packages();
    } else {
        spec = props;
    }

    let report = collect::collect_report(&spec, &options);
    let rendered = format::render(report, &options);
    print!("{}", rendered);
}
