use std::fmt::{Display, Formatter, Result as FmtResult};
/**
 * 由于没有唯一值, 只能由qq群号+seqs+time组成唯一值
 */
pub struct GroupMessageId {
    pub group_id: u64,
    pub seqs: i32,
    pub time: i64,
}

impl GroupMessageId {
    pub fn new(group_id: u64, seqs: i32, time: i64) -> GroupMessageId {
        GroupMessageId {
            group_id,
            seqs,
            time,
        }
    }

    pub fn from_bridge_message_id(bridge_message_id: &str) -> GroupMessageId {
        let splits: Vec<&str> = bridge_message_id.split('|').collect();
        println!("{:?}", splits);
        println!("{:?}", splits[1]);
        println!("{:?}", splits[2]);
        println!("{:?}", splits[3]);
        let group_id: u64 = splits[1].parse::<u64>().unwrap();
        let seqs: i32 = splits[2].parse::<i32>().unwrap();
        let time: i64 = splits[3].parse::<i64>().unwrap();
        GroupMessageId {
            group_id,
            seqs,
            time,
        }
    }
}

impl Display for GroupMessageId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "|{}|{}|{}|", self.group_id, self.seqs, self.time)
    }
}

#[test]
fn test() {
    //     reply_seq: 6539,
    //     sender: 243249439,
    //     time: 1678267174,
    let i1 = GroupMessageId::new(243249439, 6539, 1678267174);
    assert_eq!(
        i1.to_string(),
        format!("|{}|{}|{}|", 243249439, 6539, 1678267174)
    );
    println!("{}", i1.to_string());

    let i2 = GroupMessageId::from_bridge_message_id(i1.to_string().as_str());
    println!("{}", i2.to_string());

    assert_eq!(i1.to_string(), i2.to_string(),);
}
