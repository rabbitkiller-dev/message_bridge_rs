use reqwest::Client;
use std::io::Cursor;
use std::{fs::File, io::Write};

#[test]
fn test_mirai_send_group_message() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut f = File::create("71919445a77c9076e3915da81028a305.webp").unwrap();
            let client = reqwest::Client::new();
            let mut stream = client.get("https://cdn.discordapp.com/avatars/724827488588660837/71919445a77c9076e3915da81028a305.webp?size=1024")
            .send().await.unwrap();
            let mut a = Cursor::new(stream.bytes().await.unwrap());
            std::io::copy(&mut a, &mut f);
        })
}

#[test]
fn test_reqwest_download_menitype() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // let mut f = File::create("71919445a77c9076e3915da81028a305.webp").unwrap();
            let client = reqwest::Client::new();
            let url = "http://gchat.qpic.cn/gchatpic_new/1261972160/518986671-3123968978-4B7951A1D35B974B288EAC20C09033B4/0?term=2";
            let stream = client.get(url)
            .send().await.unwrap();
            let content_type = stream.headers().get(reqwest::header::CONTENT_TYPE);
            println!("{:?}", content_type);
            if let Some(value) = content_type {
                let mine = value.to_str().unwrap().parse::<mime::Mime>().unwrap();
                let ext = match mime_guess::get_mime_extensions(&mine) {
                    Some(exts) => {
                        println!("exts: {:?}", exts);
                        exts.get(0).unwrap().to_string()
                    },
                    None => {
                        mine.subtype().to_string()
                    }
                };
                println!(".ext {:?}", ext);
                let file_name = format!("{:?}.{}", md5::compute(url.as_bytes()), ext);
                let mut f = File::create(file_name).unwrap();
                let mut a = Cursor::new(stream.bytes().await.unwrap());
                std::io::copy(&mut a, &mut f).unwrap();
            }
        })
}

#[test]
fn test_path() {
    use std::path::{self, Path};
    let name = "23403b7883ae191a770a022e5d30b221";
    let ext = ".jpe";
    println!("{}{}", name, ext);
    let a = Path::new("cache").join("config.json");
    // let a = path::absolute(a).unwrap();
    println!("{:?}", a);
}

#[test]
fn test_path2() {
    use std::path::Path;
    let path = Path::new("cache").join("xxx.jpe");
    println!("1: {:?}", path);
    let path = path.to_str().unwrap().to_string();
    println!("2: {:?}", path);
    let path = Path::new(&path);
    println!("3: {:?}", path);

    let path = "cache\\831b2596d4466add31064ea593811ccc.jpe";
    let path = Path::new(path);
    println!("{:?}", path);
    let path = &"cache\\831b2596d4466add31064ea593811ccc.jpe".to_string();
    let path = Path::new(path);
    println!("{:?}", path);
}
