
use std::io::{Read, Write};
use std::fs::OpenOptions;
use std::collections::HashMap;
use serde_json::{from_str, to_string};

const BIND_MAP_PATH: &str = "./data/BindMap.json";

/// 绑定映射
pub mod bind_map {
    use crate::bridge_data::*;

    /// 尝试获取映射
    /// - `user` 目标用户；桥内通用的 user name
    pub fn get_bind(user: &str) -> Option<String> {
        let map = load();
        if let Some(u) = map.get(user) {
            return Some(u.clone());
        }
        None
    }

    /// 添加映射
    /// - `user1`, `user2` 一对映射；桥内通用的 user name
    pub fn add_bind(user1: &str, user2: &str) {
        let mut map = load();
        let len = map.len();

        if !map.contains_key(user1) {
            map.insert(user1.to_string(), user2.to_string());
        }
        if !map.contains_key(user2) {
            map.insert(user2.to_string(), user1.to_string());
        }

        if len < map.len() {
            save(&map);
        }
    }

    /// 指定一对用户删除映射
    /// - `user1`, `user2` 一对映射；移除映射需成对操作；桥内通用的 user name
    pub fn rm_bind_pair(user1: &str, user2: &str) {
        let mut map = load();
        let len = map.len();

        map.retain(|u1, u2|
            !((u1 == user1 && u2 == user2) || (u1 == user2 && u2 == user1)));
        if len > map.len() {
            save(&map);
        }
    }

    /// 指定用户删除其所有关联映射
    /// - `user` 目标用户；桥内通用的 user name
    pub fn rm_user_all_bind(user: &str) {
        let mut map = load();
        let len = map.len();

        map.retain(|u1, u2| !(u1 == user || u2 == user));
        if len > map.len() {
            save(&map);
        }
    }

    /// 读取，加载本地数据
    /// TODO 构建上下文，减少侵入
    fn load() -> HashMap<String, String> {
        let mut json = String::new();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(BIND_MAP_PATH);
        match file {
            Ok(mut f) => {
                if let Err(e) = f.read_to_string(&mut json) {
                    println!("Can not read file({}); {:#?}", BIND_MAP_PATH, e);
                }
            }
            Err(e) => println!("Can not open/create data file({}); {:#?}", BIND_MAP_PATH, e),
        };

        if !json.is_empty() {
            match from_str(&json.as_str()) {
                Ok(map) => {
                    return map;
                }
                Err(e) => println!("BindMap load fail, data can not be parsed; {:#?}", e),
            }
        }
        HashMap::new()
    }

    /// 数据写入本地
    /// TODO 异步读写
    fn save(data: &HashMap<String, String>) {
        let raw: String;
        match to_string(data) {
            Ok(json) => {
                raw = json;
            }
            Err(e) => {
                println!("Fail to parse BindMap to JSON; {:#?}", e);
                return;
            }
        }

        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(BIND_MAP_PATH);
        match file {
            Ok(mut f) => {
                if let Err(e) = f.write_all(raw.as_bytes()) {
                    println!("Can not write to file({}); {:#?}", BIND_MAP_PATH, e);
                }
            }
            Err(e) => println!("Can not open/create data file({}); {:#?}", BIND_MAP_PATH, e),
        };
    }

    #[cfg(test)]
    mod ts_bind_map {
        use chrono::Local;
        use serde_json::json;
        use crate::bridge_data::bind_map::*;

        #[test]
        fn add() {
            let st = Local::now().timestamp_millis();
            add_bind("dong", "6uopdong");
            add_bind("abc", "123");
            add_bind("aaa", "bbb");
            add_bind("ccc", "aaa");
            add_bind("ddd", "bbb");
            let et = Local::now().timestamp_millis();
            println!("{} - {} = {}", et, st, et - st);
        }

        #[test]
        fn get() {
            let st = Local::now().timestamp_millis();
            let u1 = "dong";
            match get_bind(u1) {
                None => println!("{} no mapping", u1),
                Some(u2) => println!("{} map to {}", u1, u2),
            }

            let u1 = "111";
            match get_bind(u1) {
                None => println!("{} no mapping user", u1),
                Some(u2) => println!("{} map to {}", u1, u2),
            }
            let et = Local::now().timestamp_millis();
            println!("{} - {} = {}", et, st, et - st);
        }

        #[test]
        fn rm() {
            rm_bind_pair("abc", "123");
            rm_bind_pair("aaa", "bbb");
        }

        #[test]
        fn rm_all() {
            rm_user_all_bind("aaa");
        }

        #[test]
        fn open_file() {
            {// truncate, but do not write
                let file = OpenOptions::new()
                    .truncate(true)
                    .write(true)
                    .create(true)
                    .open(BIND_MAP_PATH);
                match file {
                    Ok(_) => println!("Open file success."),
                    Err(e) => println!("Can not open/create data file({}); {:#?}", BIND_MAP_PATH, e),
                };
            }
            // try read context
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(BIND_MAP_PATH);
            match file {
                Ok(mut f) => {
                    let mut json = String::new();
                    match f.read_to_string(&mut json) {
                        Ok(_) => println!("context: {}", json),
                        Err(e) => println!("Can not read file({}); {:#?}", BIND_MAP_PATH, e),
                    }
                }
                Err(e) => println!("Can not open/create data file({}); {:#?}", BIND_MAP_PATH, e),
            };
        }

    }
}
