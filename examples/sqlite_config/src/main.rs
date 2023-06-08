use rbatis::RBatis;
use rbdc_sqlite::driver::SqliteDriver;

mod store;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let rb = RBatis::new();
    rb.link(SqliteDriver{},  "./sqlite.db")
    .await
    .expect("[abs_admin] rbatis pool init fail!");
    let mut tx = rb.acquire_begin().await.unwrap();
    
    let t = store::BizActivity {
        id: Some("2".into()),
        name: Some("2".into()),
        pc_link: Some("2".into()),
        h5_link: Some("2".into()),
        pc_banner_img: None,
        h5_banner_img: None,
        sort: None,
        status: Some(2),
        remark: Some("2".into()),
        version: Some(1),
        delete_flag: Some(1),
    };
    
    store::BizActivity::insert(&mut tx, &t).await.unwrap();
    tx.commit().await.unwrap();
    tx.rollback().await.unwrap();
}
