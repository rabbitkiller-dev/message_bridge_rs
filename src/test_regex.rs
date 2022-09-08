use reqwest::Client;
use std::io::Cursor;
use std::{fs::File, io::Write};

#[test]
fn test_mirai_send_group_message() {
    let url = "https://cdn.discordapp.com/avatars/724827488588660837/71919445a77c9076e3915da81028a305.webp?size=1024";
    url.replace(".webp?size=1024", ".png?size=30");
}
