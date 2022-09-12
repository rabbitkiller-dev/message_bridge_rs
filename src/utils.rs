use js_sandbox::{AnyError, Script};
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::io::Cursor;
// use std::{fs::File, io::Write};
use std::path;
use tokio::fs::{self, File};

pub async fn download_and_cache(url: &str) -> Result<String, reqwest::Error> {
    init().await;
    let client = reqwest::Client::new();
    let stream = client.get(url).send().await?;
    let url_md5 = format!("{:x}", md5::compute(url.as_bytes()));

    let content_type = stream.headers().get(reqwest::header::CONTENT_TYPE);

    let ext = match content_type {
        Some(value) => {
            let mine = value.to_str().unwrap().parse::<mime::Mime>().unwrap();
            let ext = match mime_guess::get_mime_extensions(&mine) {
                Some(exts) => exts.last().unwrap().to_string(),
                None => mine.subtype().to_string(),
            };
            format!(".{}", ext)
        }
        None => String::new(),
    };
    let file_name = format!("{}{}", url_md5, ext);
    let file_name = path::Path::new("cache").join(file_name);
    let mut f = File::create(file_name.clone()).await.unwrap();
    let mut a = Cursor::new(stream.bytes().await.unwrap());
    tokio::io::copy(&mut a, &mut f).await.unwrap();

    Ok(file_name.to_str().unwrap().to_string())
}

pub async fn init() {
    if !std::path::Path::new("cache").exists() {
        if let Err(err) = tokio::fs::create_dir("cache").await {
            println!("初始化cache目录失败");
            println!("{:?}", err);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarkdownAst {
    // 文本
    Plain { text: String },
    // 桥@成员
    At { username: String },
    // @dc成员
    AtInDiscordUser { id: String },
}

/**
 * 将dc和qq消息进行解析
 */
pub fn parser_message(content: &str) -> Vec<MarkdownAst> {
    let str = std::fs::read_to_string("./mde.js").unwrap();
    let mut script = Script::from_string(str.as_str()).unwrap();

    let mut result: Vec<MarkdownAst> = script.call("parserBridgeMessage", &content).unwrap();

    if let Some(ast) = result.last() {
        if let MarkdownAst::Plain { text } = ast {
            if (text.eq("\n")) {
                result.remove(result.len() - 1);
            }
        }
    }
    if let Some(ast) = result.last() {
        if let MarkdownAst::Plain { text } = ast {
            if (text.eq("\n")) {
                result.remove(result.len() - 1);
            }
        }
    }

    result
}

/**
 * 测试parser_message
 */
#[test]
pub fn test_() {
    let vec = parser_message("<@724827488588660837>");
    println!("{:?}", vec);
    let vec = parser_message(
        r#"@[DC] 6uopdong#4700
    !绑定 qq 1261972160"#,
    );
    println!("{:?}", vec);
}
