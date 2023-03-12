use serde::Deserialize;
use serde::Serialize;
use std::io::Cursor;
use std::path;
use tokio::fs::File;

pub async fn download_and_cache(url: &str) -> Result<String, reqwest::Error> {
    init().await;
    let client = reqwest::Client::new();
    let stream = client.get(url).send().await?;
    let url_md5 = format!("{:x}", md5::compute(url.as_bytes()));

    let content_type = stream.headers().get(reqwest::header::CONTENT_TYPE);

    let ext = match content_type {
        Some(value) => get_mine_type_ext(value.to_str().unwrap()),
        None => String::new(),
    };
    let file_name = format!("{}{}", url_md5, ext);
    let file_name = path::Path::new("cache").join(file_name);
    let mut f = File::create(file_name.clone()).await.unwrap();
    let mut a = Cursor::new(stream.bytes().await.unwrap());
    tokio::io::copy(&mut a, &mut f).await.unwrap();

    Ok(file_name.to_str().unwrap().to_string())
}

pub fn get_mine_type_ext(mime_type: &str) -> String {
    if "image/jpeg".eq(mime_type) {
        return ".jpg".to_string();
    }
    let mine = mime_type.parse::<mime::Mime>().unwrap();
    let ext = match mime_guess::get_mime_extensions(&mine) {
        Some(exts) => exts.first().unwrap().to_string(),
        None => mine.subtype().to_string(),
    };
    format!(".{}", ext)
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
    // Common: 文本
    Plain {
        text: String,
    },
    // Common: @成员
    At {
        username: String,
    },
    // DC: @成员
    DiscordAtUser {
        id: String,
    },
    // DC: @所有人
    DiscordAtEveryone {},
    // DC: @频道所有人
    DiscordAtHere {},
    // DC: emoji图片
    DiscordEmoji {
        id: String,
        name: String,
        animated: bool,
    },
}

/**
 * 将dc和qq消息进行解析
 */
pub async fn parser_message(content: &str) -> Vec<MarkdownAst> {
    let client = reqwest::Client::new();
    let mut result: Vec<MarkdownAst> = client
        .post("http://localhost:3000/parse-discord-markdown")
        .body(content.to_string())
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .expect("请求解析discord消息服务失败")
        .json()
        .await
        .expect("解析discord消息回传解析json失败");

    if let Some(ast) = result.last() {
        if let MarkdownAst::Plain { text } = ast {
            if text.eq("\n") {
                result.remove(result.len() - 1);
            }
        }
    }
    if let Some(ast) = result.last() {
        if let MarkdownAst::Plain { text } = ast {
            if text.eq("\n") {
                result.remove(result.len() - 1);
            }
        }
    }

    result
}

#[test]
fn test_send_post_parse_discord_message() {
    let message = r#"@[DC] 6uopdong#4700
    !绑定 qq 1261972160"#;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            println!("发送");
            let client = reqwest::Client::new();
            let resp: Vec<MarkdownAst> = client
                .post("http://localhost:3000/parse-discord-markdown")
                .body(message)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            println!("{:?}", resp);
        })
}

#[test]
fn test2() {
    println!("{:?}", "zhangsan");
    println!("{}", "zhangsan");
}
#[test]
fn test3() {
    let r = "@rabbitBot2".strip_prefix("@").unwrap();
    println!("{:?}", r);
}

#[test]
fn test4() {
    let mine = "image/jpeg".parse::<mime::Mime>().unwrap();
    println!("{:?}", mine);
    match mime_guess::get_mime_extensions(&mine) {
        Some(exts) => {
            println!("Some: {:?}", exts);
            println!("Some: {}", exts.first().unwrap().to_string());
        }
        None => {
            println!("None: {}", mine.subtype().to_string());
        }
    };
}
