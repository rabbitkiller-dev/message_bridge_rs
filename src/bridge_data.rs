///! 定义桥的数据结构，读写方法

use {
    crate::bridge::{BridgeClientPlatform as BCP, User},
    std::{
        fs::{create_dir_all, OpenOptions},
        io::{Read, Write},
        path,
        path::Path,
    },
};

///! 定义绑定映射
pub mod bind_map {
    use super::*;

    /// 平台枚举, unique_id, display_id
    type BindKey = (u64, u64, u64);
    type BindData = Vec<(BindKey, BindKey)>;

    const BIND_MAP_PATH: &str = "./data/BindMap.json";

    /// 尝试获取映射
    /// # 参数
    /// - `user` 目标用户
    /// - `to_platform` 指向绑定的平台
    /// # 返回
    /// 含部分有效字段的 User: platform, unique_id, display_id
    pub fn get_bind(user: &User, to_platform: BCP) -> Option<User> {
        // 有必要自绑定吗？
        if to_platform == user.platform {
            return None;
        }
        let pp = user.platform as u64 | to_platform as u64;
        let data = load();

        for (a @ (p1, u1, d1), b @ (p2, u2, d2)) in data.iter() {
            if (p1 | p2) == pp {
                let f: Option<BindKey> =
                    if *u1 == user.unique_id || *d1 == user.display_id {
                        Some(*b)
                    } else if *u2 == user.unique_id || *d2 == user.display_id {
                        Some(*a)
                    } else {
                        None
                    };
                if let Some((p, u, d)) = f {
                    return Some(User {
                        name: "".to_string(),
                        avatar_url: None,
                        platform_id: 0,
                        platform: BCP::by(p).unwrap(),
                        display_id: d,
                        unique_id: u,
                    });
                }
            }
        }
        None
    }

    /// 添加映射
    /// - `user1`, `user2` 一对映射
    pub fn add_bind(user1: &User, user2: &User) -> bool {
        let p = (user1.platform as u64, user2.platform as u64);
        let mut data = load();
        let mut add = true;
        let pp = p.0 | p.1;

        for ((p1, u1, d1), (p2, u2, d2)) in data.iter() {
            if (p1 | p2) == pp {
                if (*u1 == user1.unique_id && *u2 == user2.unique_id) ||
                    (*u2 == user1.unique_id && *u1 == user2.unique_id) {
                    add = false;
                    break;
                }
                if (*d1 == user1.display_id && *d2 == user2.display_id) ||
                    (*d2 == user1.display_id && *d1 == user2.display_id) {
                    add = false;
                    break;
                }
            }
        }

        if add {
            data.push(((p.0, user1.unique_id, user1.display_id), (p.1, user2.unique_id, user2.display_id)));
            return save(&data);
        }
        false
    }

    /// 指定一对用户删除映射
    /// - `user1`, `user2` 一对映射；移除映射需成对操作
    pub fn rm_bind_pair(user1: &User, user2: &User) {
        let mut data = load();
        let len = data.len();
        let pp = user1.platform as u64 | user2.platform as u64;

        data.retain(|((p1, u1, d1), (p2, u2, d2))| {
            if (p1 | p2) == pp {
                !((*u1 == user1.unique_id && *u2 == user2.unique_id) ||
                    (*u1 == user2.unique_id && *u2 == user1.unique_id) ||
                    (*d1 == user1.display_id && *d2 == user2.display_id) ||
                    (*d1 == user2.display_id && *d2 == user1.display_id))
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

        data.retain(|((p1, u1, d1), (p2, u2, d2))|
            if (p1 | p2) & p > 0 {
                !(*u1 == user.unique_id || *u2 == user.unique_id ||
                    *d1 == user.display_id || *d2 == user.display_id)
            } else {
                true
            });
        if len > data.len() {
            save(&data);
        }
    }

    /// 初始化数据文件目录
    /// # return
    /// 检查与创建是否成功
    fn init_dir() -> bool {
        let dat_dir = Path::new(BIND_MAP_PATH).parent().unwrap();
        if dat_dir.as_os_str().is_empty() || dat_dir.exists() {
            return true;
        }
        if let Err(e) = create_dir_all(dat_dir) {
            println!("目录'{}'创建失败！{:#?}", dat_dir.to_str().unwrap(), e);
            return false;
        }
        true
    }

    /// 读取，加载本地数据
    fn load() -> BindData {
        if !init_dir() {
            return BindData::new();
        }
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
            match serde_json::from_str::<BindData>(&json.as_str()) {
                Ok(mut data) => {
                    // 删除无平台映射
                    data.retain(|((p1, ..), (p2, ..))| *p1 > 0 && *p2 > 0);
                    return data;
                }
                Err(e) => println!("BindMap load fail, data can not be parsed; {:#?}", e),
            }
        }
        BindData::new()
    }

    /// 数据写入本地
    /// TODO 异步读写
    fn save(data: &BindData) -> bool {
        if !init_dir() {
            return false;
        }
        let raw: String;
        match serde_json::to_string(data) {
            Ok(json) => {
                raw = json;
            }
            Err(e) => {
                println!("Fail to parse BindMap to JSON; {:#?}", e);
                return false;
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
                    return false;
                }
            }
            Err(e) => {
                println!("Can not open/create data file({}); {:#?}", BIND_MAP_PATH, e);
                return false;
            }
        };
        true
    }

    #[cfg(test)]
    mod ts_bind_map {
        use {
            chrono::Local,
            crate::{
                bridge::{
                    BridgeClientPlatform::*,
                    User,
                },
                bridge_data::bind_map::*,
            },
        };

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
                platform: Discord,
            };
            let u2 = User {
                name: "".to_string(),
                avatar_url: None,
                unique_id: 0,
                display_id: 0,
                platform_id: 0,
                platform: QQ,
            };
            let st = Local::now().timestamp_millis();
            match get_bind(&u1, QQ) {
                None => println!("{} no mapping", u1.unique_id),
                Some(u2) => println!("{} map to {}", u1.unique_id, u2.unique_id),
            }
            match get_bind(&u2, Discord) {
                None => println!("{} no mapping user", u2.unique_id),
                Some(u3) => println!("{} map to {}", u2.unique_id, u3.unique_id),
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
                    platform: Discord,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 222_222,
                    display_id: 222,
                    platform_id: 222,
                    platform: QQ,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 333_333,
                    display_id: 333,
                    platform_id: 333,
                    platform: Discord,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 444_444,
                    display_id: 444,
                    platform_id: 444,
                    platform: QQ,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 555_555,
                    display_id: 555,
                    platform_id: 555,
                    platform: Discord,
                },
                User {
                    name: emp(),
                    avatar_url: None,
                    unique_id: 666_666,
                    display_id: 666,
                    platform_id: 666,
                    platform: Discord,
                },
            ]
        }
    }
}
