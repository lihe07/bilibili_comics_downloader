use log::info;
use log::info;

pub async fn login_info() {
    let config = Config::load();
    if let Some(user_info) = network::get_user_info(&config).await {
        info!("登录信息有效！");
        log.info(format!("用户名：{}", user_info.name));
        log.info(format!("漫币余额：{}", user_info.coin));
    } else {
        log.info("登录信息无效或未登录！");
    }
}

pub enum LoginMethod {
    SESSDATA(String),
    QRCODE,
}

pub async fn login(method: LoginMethod) {
    let mut config = Config::load();
    match method {
        LoginMethod::SESSDATA(sessdata) => {
            config.sessdata = sessdata;
            if let Some(user_info) = network::get_user_info(&config).await {
                log.info("登录信息有效！");
                log.info(format!("用户名：{}", user_info.name));
                log.info(format!("漫币余额：{}", user_info.coin));
                config.save();
            } else {
                log.error("登录信息无效！");
            }
        }
        LoginMethod::QRCODE => {
            let (qr_data, oauth) = network::get_qr_data(&config).await;
            let code = QrCode::new(&qr_data).unwrap();
            let image = code
                .render::<qrcode::render::unicode::Dense1x2>()
                .dark_color(qrcode::render::unicode::Dense1x2::Dark)
                .light_color(qrcode::render::unicode::Dense1x2::Light)
                .build();
            println!("{}", image);
            log.success("二维码已生成，请扫描二维码登录");
            log.info(format!("如果显示错误，请手动访问：{}", qr_data));
            log.loading("等待扫描...");
            let mut last_status = "NotScan";
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                match network::check_qr_status(&config, oauth.clone()).await {
                    network::QRStatus::NotScan => {
                        if last_status != "NotScan" {
                            log.done();
                            log.loading("等待扫描...");
                        }
                        last_status = "NotScan";
                    }
                    network::QRStatus::Scanning => {
                        if last_status != "Scanning" {
                            log.done();
                            log.loading("等待确认...");
                        }
                        last_status = "Scanning";
                    }
                    network::QRStatus::Complete(sessdata) => {
                        log.done();
                        log.success("登录成功！");

                        config.sessdata = sessdata;
                        config.save();
                        let user_info = network::get_user_info(&config).await.unwrap();
                        log.info("登录信息有效！");
                        log.info(format!("用户名：{}", user_info.name));
                        log.info(format!("漫币余额：{}", user_info.coin));

                        return;
                    }
                    network::QRStatus::Invalid => {
                        log.error("二维码已失效，请重新运行程序");
                        return;
                    }
                }
            }
        }
    }
}
