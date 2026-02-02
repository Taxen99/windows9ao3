use std::{collections::HashMap, env, fs};

use arcthing::{
    config::{BuildOptions, Config},
    deploy,
    resource_resolver::{self, get_resource_path},
};

// struct Arg {
//     name: &'static str,
// }

struct ArgParser {
    // args: Vec<Arg>,
}

impl ArgParser {
    fn parse() -> HashMap<String, String> {
        let mut args = env::args().peekable();
        let mut map: HashMap<String, String> = HashMap::new();
        loop {
            let arg = args.next();
            match arg {
                Some(arg) => {
                    if let Some(arg) = arg.strip_prefix("--") {
                        if let Some(value) = args.peek() {
                            if value.strip_prefix("--").is_some() {
                                map.insert(arg.into(), "".into());
                                continue;
                            } else {
                                map.insert(arg.into(), value.clone());
                            }
                        } else {
                            map.insert(arg.into(), "".into());
                            break;
                        }
                    }
                }
                None => break,
            }
        }
        map
    }
}

pub fn main() {
    // let arg_parser = ArgParser {
    //     args: vec![
    //         Arg {
    //             name: "open-window",
    //         },
    //         Arg { name: "deploy" },
    //     ],
    // };
    let parsed = ArgParser::parse();
    let mut build_opt: BuildOptions = Default::default();
    let mut deploy = false;
    for (arg, value) in parsed {
        //
        match arg.as_str() {
            "init" => {
                build_opt.initial_window = Some(value.parse::<u64>().expect("invalid window id"))
            }
            "deploy" => {
                deploy = true;
            }
            "noboot" => {
                build_opt.bypass_boot = true;
            }
            _ => (),
        }
    }
    let config: Config = ron::from_str(&fs::read_to_string("config.ron").unwrap()).unwrap();
    let res = config.build(build_opt);
    let _ = fs::create_dir("output");
    let resolved_res = resource_resolver::resolve_resources(&res, |kind, res| {
        format!("../{}", get_resource_path(kind, res).to_str().unwrap())
    });
    {
        let html = format!(
            r##"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Document</title>
                <link rel="stylesheet" href="style.css">
            </head>
            <body>
                {}
            </body>
            </html>
            "##,
            &resolved_res.html
        );
        fs::write("output/style.css", &resolved_res.css).unwrap();
        fs::write("output/index.html", html).unwrap();
    }
    if deploy {
        let ao3_res = deploy::deploy(&res);
        fs::write("output/ao3.css", &ao3_res.css).unwrap();
        fs::write("output/ao3.html", &ao3_res.html).unwrap();
    } else {
        let _ = fs::remove_file("output/ao3.css");
        let _ = fs::remove_file("output/ao3.html");
    }
    // fs::write("output/ao3.html", res.ao3_html).unwrap();
    // fs::write("output/ao3.css", res.ao3_css).unwrap();
}
