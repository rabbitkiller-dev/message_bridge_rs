use reqwest::Client;
use std::io::Cursor;
use std::{fs::File, io::Write};

#[test]
fn test_mirai_send_group_message() {
    let url = "https://cdn.discordapp.com/avatars/724827488588660837/71919445a77c9076e3915da81028a305.webp?size=1024";
    url.replace(".webp?size=1024", ".png?size=30");
}
/**
 * 测试正则裁剪和替换文本内容
 */
#[test]
fn test() {
    let mut chain: Vec<String> = vec![];
    let splitTo = "#|x-x|#".to_string();
    let reg_at_user = regex::Regex::new(r"@\[DC\] ([^\n^#^@]+)?#(\d\d\d\d)").unwrap();
    let mut text = r#"test qq 1 @[DC] 6uopdong#4700你看看@[DC] rabbitkiller#7372"#.to_string();
    // let caps = reg_at_user.captures(text);
    while let Some(caps) = reg_at_user.captures(text.as_str()) {
        println!("{:?}", caps);
        let from = caps.get(0).unwrap().as_str();
        let name = caps.get(1).unwrap().as_str();
        let disc = caps.get(2).unwrap().as_str();

        let result = text.replace(from, &splitTo);
        let splits: Vec<&str> = result.split(&splitTo).collect();
        let prefix = splits.get(0).unwrap();
        chain.push(prefix.to_string());
        if let Some(fix) = splits.get(1) {
            text = fix.to_string();
        }
    }
    chain.push(text.to_string());
    println!("{:?}", chain);
}
/**
 * 测试正则裁剪和替换文本内容 qq
 */
#[test]
fn test_qq() {
    let mut chain: Vec<String> = vec![];
    let splitTo = "#|x-x|#".to_string();
    let reg_at_user = regex::Regex::new(r"@\[QQ\] ([^\n^@]+)\(([0-9]+)\)").unwrap();
    let mut text = r#"test qq 1 @[QQ] sanda(243249439)你看看@[QQ] sanda(243249439)"#.to_string();
    // let caps = reg_at_user.captures(text);
    while let Some(caps) = reg_at_user.captures(text.as_str()) {
        println!("{:?}", caps);
        let from = caps.get(0).unwrap().as_str();
        let name = caps.get(1).unwrap().as_str();
        let disc = caps.get(2).unwrap().as_str();

        let result = text.replace(from, &splitTo);
        let splits: Vec<&str> = result.split(&splitTo).collect();
        let prefix = splits.get(0).unwrap();
        chain.push(prefix.to_string());
        if let Some(fix) = splits.get(1) {
            text = fix.to_string();
        }
    }
    chain.push(text.to_string());
    println!("{:?}", chain);
}

/**
 * dc
 */
#[test]
fn test2() {
    let text = r#"test qq 1 @[DC] 6uopdong#4700你看看@[DC] rabbitkiller#7372"#.to_string();
    let splits: Vec<&str> = text.split(" ").collect();
    let mut reply_content: Vec<String> = vec![];
    for sp in splits {
        reply_content.push(format!("> {}\n", sp));
    }
    let mut content = vec![];
    content.push("测试看看".to_string());
    // result.push(value)
    reply_content.append(&mut content);
    println!("{:?}", reply_content);
}
