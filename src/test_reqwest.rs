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
