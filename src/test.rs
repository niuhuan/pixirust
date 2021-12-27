#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::{Client, Token};
    use std::sync::{Mutex, MutexGuard};

    // 懒加载, 维护全局token
    lazy_static::lazy_static! {
        static ref TOKEN:Mutex<Token> = Mutex::<Token>::new(
            Token{
                access_token: String::default(),
                expires_in: 0,
                token_type: String::default(),
                scope: String::default(),
                refresh_token: String::default(),
            }
        );
    }

    /// 打印结果
    fn print<T: serde::Serialize>(result: Result<T>) {
        match result {
            Ok(data) => println!("{}", serde_json::to_string(&data).unwrap()),
            Err(err) => panic!("{}", err),
        }
    }

    /// 生成一个没有认证登录过的客户端
    fn no_auth_client() -> Client {
        Client::new_agent_free()
    }

    /// 生成登录链接
    #[tokio::test]
    async fn test_login_url() {
        println!(
            "{}",
            serde_json::to_string(&no_auth_client().create_login_url()).unwrap()
        );
    }

    /// 使用登录链接登录后, 客户端oauth
    #[tokio::test]
    async fn test_load_token_by_code() {
        let token = no_auth_client()
            .load_token_by_code("code".to_string(), "verify".to_string())
            .await
            .unwrap();
        write_token(token, chrono::Local::now().timestamp_millis());
    }

    /// 保存token到文件
    fn write_token(token: Token, time: i64) {
        std::fs::write(
            "test_token.json",
            serde_json::to_string(&token.clone()).unwrap(),
        )
        .unwrap();
        std::fs::write("test_token_time.json", format!("{}", time)).unwrap();
    }

    fn copy(token: &mut MutexGuard<Token>, source: Token) {
        token.token_type = source.token_type;
        token.access_token = source.access_token;
        token.refresh_token = source.refresh_token;
        token.scope = source.scope;
        token.expires_in = source.expires_in;
    }

    async fn authed_client() -> Result<Client> {
        // 初始化(仅一次)
        let now = chrono::Local::now().timestamp_millis();
        let src_token: Token =
            serde_json::from_str(std::fs::read_to_string("test_token.json").unwrap().as_str())
                .unwrap();
        let time: i64 = serde_json::from_str(
            std::fs::read_to_string("test_token_time.json")
                .unwrap()
                .as_str(),
        )
        .unwrap();
        let mut token = TOKEN.lock().unwrap();
        copy(&mut token, src_token);
        drop(token);
        // 运行中, 每次请求
        let mut client = no_auth_client();
        let mut token = TOKEN.lock().unwrap();
        if token.expires_in + time < now {
            let new_token = (&client).refresh_token(&token.refresh_token).await?;
            write_token(new_token.clone(), now);
            copy(&mut token, new_token)
        }
        let result = token.access_token.clone();
        drop(token);
        client.access_token = result;
        Ok(client)
    }

    #[tokio::test]
    async fn test_raw() {
        let client = authed_client().await.unwrap();
        println!(
            "{}",
            client
                .get_from_pixiv_raw(client.illust_recommended_first_url())
                .await
                .unwrap()
        )
    }

    #[tokio::test]
    async fn test_illust() {
        let client = authed_client().await.unwrap();
        print(
            client
                .illust_from_url(
                    client.illust_rank_first_url("day_r18".to_string(), String::default()),
                )
                .await,
        )
    }

    #[tokio::test]
    async fn test_load_image() {
        match no_auth_client().load_image_data("https://i.pximg.net/c/540x540_70/img-master/img/2021/04/18/17/22/42/89233845_p0_master1200.jpg".to_string()).await {
            Ok(img_bytes) => match std::fs::write("test.jpg", img_bytes) {
                Ok(_) => println!("OK"),
                Err(err) => panic!("{}", err),
            },
            Err(err) => panic!("{}", err),
        }
    }

    #[tokio::test]
    async fn test() {
        println!("{}", chrono::Local::now().timestamp_millis())
    }
}
