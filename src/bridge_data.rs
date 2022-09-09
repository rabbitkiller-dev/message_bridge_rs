use std::fs::OpenOptions;
use std::io::{Read, Write};

use serde_json::{from_str, to_string};

use crate::bridge::User;

/// 绑定映射
pub mod bind_map {
    use crate::bridge_data::*;

    type BindKey = (u64, u64);
    type BindData = Vec<(BindKey, BindKey)>;

    const BIND_MAP_PATH: &str = "./data/BindMap.json";

    /// 尝试获取映射
    /// - `user` 目标用户
    pub fn get_bind(user: &User) -> Option<u64> {
        let pp = user.platform as u64;
        let data = load();

        for ((p1, u1), (p2, u2)) in data.iter() {
            if (p1 | p2) & pp > 0 {
                if *u1 == user.unique_id {
                    return Some(*u2);
                }
                if *u2 == user.unique_id {
                    return Some(*u1);
                }
            }
        }
        None
    }

    /// 添加映射
    /// - `user1`, `user2` 一对映射
    pub fn add_bind(user1: &User, user2: &User) {
        let p = (user1.platform as u64, user2.platform as u64);
        let mut data = load();
        let mut add = true;
        let pp = p.0 | p.1;

        for ((p1, u1), (p2, u2)) in data.iter() {
            if (p1 | p2) == pp {
                if (*u1 == user1.unique_id && *u2 == user2.unique_id) ||
                    (*u2 == user1.unique_id && *u1 == user2.unique_id) {
                    add = false;
                    break;
                }
            }
        }

        if add {
            data.push(((p.0, user1.unique_id), (p.1, user2.unique_id)));
            save(&data)
        }
    }

    /// 指定一对用户删除映射
    /// - `user1`, `user2` 一对映射；移除映射需成对操作
    pub fn rm_bind_pair(user1: &User, user2: &User) {
        let mut data = load();
        let len = data.len();
        let pp = user1.platform as u64 | user2.platform as u64;

        data.retain(|((p1, u1), (p2, u2))| {
            if (p1 | p2) == pp {
                !((*u1 == user1.unique_id && *u2 == user2.unique_id) ||
                    (*u1 == user2.unique_id && *u2 == user1.unique_id))
            } else {
                true
            }
        });
        if len > data.len() {
            save(&data);
        }
    }

    /// 指定用户删除其所有关联映射
    /// - `user` 目标用户
    pub fn rm_user_all_bind(user: &User) {
        let mut data = load();
        let len = data.len();
        let p = user.platform as u64;

        data.retain(|((p1, u1), (p2, u2))|
            if (p1 | p2) & p > 0 {
                !(*u1 == user.unique_id || *u2 == user.unique_id)
            } else {
                true
            });
        if len > data.len() {
            save(&data);
        }
    }

    /// 读取，加载本地数据
    /// TODO 构建上下文，减少侵入
    fn load() -> BindData {
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
            match from_str::<BindData>(&json.as_str()) {
                Ok(mut data) => {
                    // 删除无平台映射
                    data.retain(|((p1, _), (p2, _))| *p1 > 0 && *p2 > 0);
                    return data;
                }
                Err(e) => println!("BindMap load fail, data can not be parsed; {:#?}", e),
            }
        }
        BindData::new()
    }

    /// 数据写入本地
    /// TODO 异步读写
    fn save(data: &BindData) {
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

        use crate::bridge::{BridgeClientPlatform, User};
        use crate::bridge_data::bind_map::*;

        #[test]
        fn add() {
            let uls = get_users();
            let st = Local::now().timestamp_millis();
            add_bind(&uls[0], &uls[1]);
            add_bind(&uls[2], &uls[3]);
            add_bind(&uls[4], &uls[1]);
            add_bind(&uls[5], &uls[2]);
            let et = Local::now().timestamp_millis();
            println!("add 4 mapping: {}ms", et - st);
        }

        #[test]
        fn get() {
            let u1 = User {
                name: "".to_string(),
                avatar_url: None,
                unique_id: 111_111,
                display_id: 111,
                platform_id: 111,
                platform: BridgeClientPlatform::Discord,
            };
            let u2 = User {
                name: "".to_string(),
                avatar_url: None,
                unique_id: 0,
                display_id: 0,
                platform_id: 0,
                platform: BridgeClientPlatform::QQ,
            };
            let st = Local::now().timestamp_millis();
            match get_bind(&u1) {
                None => println!("{} no mapping", u1.unique_id),
                Some(u2) => println!("{} map to {}", u1.unique_id, u2),
            }
            match get_bind(&u2) {
                None => println!("{} no mapping user", u2.unique_id),
                Some(u3) => println!("{} map to {}", u2.unique_id, u3),
            }
            let et = Local::now().timestamp_millis();
            println!("get 2 mapping: {}ms", et - st);
        }

        #[test]
        fn rm() {
            let uls = get_users();
            rm_bind_pair(&uls[0], &uls[1]);
            rm_bind_pair(&uls[2], &uls[3]);
        }

        #[test]
        fn rm_all() {
            let uls = get_users();
            rm_user_all_bind(&uls[1]);
        }

        fn get_users() -> Vec<User> {
            fn emp() -> String {
                "".to_string()
            }
            vec![
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 111_111,
                    display_id: 111,
                    platform_id: 111,
                    platform: BridgeClientPlatform::Discord,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 222_222,
                    display_id: 222,
                    platform_id: 222,
                    platform: BridgeClientPlatform::QQ,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 333_333,
                    display_id: 333,
                    platform_id: 333,
                    platform: BridgeClientPlatform::Discord,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 444_444,
                    display_id: 444,
                    platform_id: 444,
                    platform: BridgeClientPlatform::QQ,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 555_555,
                    display_id: 555,
                    platform_id: 555,
                    platform: BridgeClientPlatform::Discord,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 666_666,
                    display_id: 666,
                    platform_id: 666,
                    platform: BridgeClientPlatform::Discord,
                },
            ]
        }
    }
}
