use crate::api::ali_api::{
    add_aliyun_dns_record, delete_aliyun_dns, query_aliyun_dns_list,
    query_aliyun_dns_operation_list, query_aliyun_domain_list,
};
use crate::api::model::dns_operate::RecordLog;
use crate::configs::config::LICENCE;
use crate::gui::components::footer::footer;
use crate::gui::components::header::header;
use crate::gui::components::types::my_modal::MyModal;
use crate::gui::model::domain::DomainName;
use crate::gui::model::form::{AddDnsField, AddDomainField};
use crate::gui::pages::domain::{add_domain_page, domain_page};
use crate::gui::pages::domain_dns_record::{add_dns_record, dns_record};
use crate::gui::pages::help::help;
use crate::gui::pages::names::Page;
use crate::gui::pages::types::settings::SettingsPage;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::gui::types::message::Message;
use crate::model::dns_record_response::Record;
use crate::translations::types::language::Language;
use crate::translations::types::locale::Locale;
use crate::{Config, StyleType};
use iced::keyboard::Key;
use iced::widget::{text, Column, Container, Text};
use iced::window::{Id, Position};
use iced::Event::Window;
use iced::{
    application, keyboard, window, Element, Font, Length, Pixels, Point, Settings, Size,
    Subscription, Task, Theme,
};
use log::info;
use std::fmt::format;
use std::process;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct DomainManager {
    /// 应用程序的配置：设置、窗口属性、应用程序名称
    pub config: Config,
    /// 当前主题
    pub theme: Theme,
    pub domain_names: Vec<DomainName>,
    /// 当前页面
    pub current_page: Page,
    pub current_domain_name: Option<DomainName>,
    pub add_domain_field: AddDomainField,
    pub last_page: Option<Page>,
    /// 查询进行中
    pub in_query: bool,
    /// dns列表
    pub dns_list: Vec<Record>, // 当前域名对应的DNS记录
    pub dns_log_list: Vec<RecordLog>, // 当前域名对应的DNS记录
    pub add_dns_form: AddDnsField,
    pub locale: Locale,
    /// 缩略图模式当前是否处于活动状态
    pub thumbnail: bool,
    /// 未读通知数
    pub unread_notifications: usize,
    pub last_opened_setting: Option<SettingsPage>,
    pub newer_release_available: Mutex<Option<bool>>,
    pub modal: Option<MyModal>,
}

impl Default for DomainManager {
    fn default() -> Self {
        let config = Config {
            name: String::from("Domain Manager"),
            description: String::from("A simple domain manager"),
            version: String::from("1.0.0"),
            author: String::from("Stanic.xyz"),
            license: LICENCE::MulanPSL2,
            domain_names: vec![],
            locale: String::from("en"),
            style_type: StyleType::Day,
            language: Language::ZH,
            color_gradient: GradientType::Mild,
        };

        // 初始化数据
        Self {
            current_page: Page::DomainPage,
            theme: Theme::Dark,
            domain_names: vec![],
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            add_dns_form: AddDnsField::default(),
            last_page: None,
            in_query: false,
            dns_list: vec![],
            dns_log_list: vec![],
            locale: Locale::Chinese,
            config,
            thumbnail: false,
            unread_notifications: 0,
            last_opened_setting: None,
            newer_release_available: Mutex::new(None),
            modal: None,
        }
    }
}

// 定义主题
impl DomainManager {
    
    /// 启动
    pub fn start(&self) -> iced::Result {
        self.locale();
        let app = application("Domain Manager", Self::update, Self::view)
            .window(window::Settings {
                size: Size::new(1080f32, 720f32), // start size
                position: Position::Default,
                min_size: Some(Size::new(320f32, 240f32)), // Some(ConfigWindow::MIN_SIZE.to_size()), // min size allowed
                max_size: Some(Size::new(1080f32, 720f32)),
                visible: true,
                resizable: true,
                decorations: true,
                transparent: true,
                exit_on_close_request: true,
                ..Default::default()
            })
            .subscription(DomainManager::keyboard_subscription)
            .subscription(DomainManager::subscription)
            .settings(Settings {
                fonts: vec![
                    include_bytes!("../../resources/fonts/subset/icons.ttf").into(),
                    include_bytes!("../../resources/fonts/full/MapleMono-NF-CN-Regular.ttf").into(),
                ],
                default_font: Font::with_name("Maple Mono NF CN"),
                default_text_size: Pixels::from(14),
                ..Default::default()
            });
        app.run()
    }

    fn locale(&self) {
        match self.locale {
            Locale::Chinese => rust_i18n::set_locale("zh_CN"),
            Locale::English => rust_i18n::set_locale("en"),
        }
    }

    pub fn new(config: Config) -> Self {
        // 初始化数据
        let domain_names = config.domain_names.clone();
        let locale: Locale = config.locale.clone().into();

        Self {
            current_page: Page::DomainPage,
            theme: Theme::Light,
            domain_names,
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            last_page: None,
            in_query: false,
            config,
            thumbnail: false,
            dns_list: vec![],
            dns_log_list: vec![],
            add_dns_form: AddDnsField::default(),
            locale,
            ..DomainManager::default()
        }
    }

    fn view(&self) -> Element<Message, StyleType> {
        // 保持锁的有效性
        let config = &self.config;
        let body = match self.current_page {
            Page::DomainPage => domain_page(self),
            Page::AddDomain => add_domain_page(self),
            Page::DnsRecord => dns_record(self),
            Page::AddRecord => add_dns_record(self),
            Page::Help => help(self),
        };

        let header = header(self);
        let footer = footer(
            false,
            config.language,
            config.color_gradient,
            config.style_type.get_extension().font,
            config.style_type.get_extension().font_headers,
            &Mutex::new(Some(true)),
        );

        Column::new()
            .push(header)
            .push(body.height(Length::Fill).width(Length::Fill))
            .push(footer)
            .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        // 这里应该按照每一个页面来处理事件响应的
        match self.current_page {
            Page::DomainPage => {}
            Page::AddDomain => {}
            Page::DnsRecord => {}
            Page::AddRecord => {}
            _ => {}
        }

        // 按照每一个事件来处理
        match message {
            Message::ChangeLocale => {
                self.locale = match self.locale {
                    Locale::Chinese => Locale::English,
                    Locale::English => Locale::Chinese,
                };
                self.locale();
                Task::none()
            }
            Message::ToggleTheme => {
                if self.theme == Theme::TokyoNightLight {
                    self.theme = Theme::SolarizedDark
                } else {
                    self.theme = Theme::TokyoNightLight
                }
                dbg!("修改主题为{}", &self.theme);
                Task::none()
            }
            // 改变当前页面
            Message::ChangePage(page) => {
                info!("Page Changed");
                let current_page = self.current_page.clone();
                self.current_page = page;
                self.update(Message::PageChanged(
                    current_page,
                    self.current_page.clone(),
                ))
            }
            Message::PageChanged(last_page, current_page) => {
                info!("页面从{}切换到{}", last_page, current_page);
                match current_page {
                    Page::DnsRecord => {
                        if self.in_query {
                            // 已经在查询中，不进行任何操作
                            Task::none()
                        } else {
                            info!("查询dns记录：domain_name:{:?}", &self.current_domain_name);
                            match &self.current_domain_name {
                                Some(domain_name) => {
                                    let name: String = domain_name.get_domain_name().into();
                                    let name_for_log_query: String =
                                        domain_name.get_domain_name().into();

                                    // 多个事件
                                    Task::batch([
                                        Task::perform(
                                            Self::handle_dns_reload(name),
                                            |dns_records| {
                                                println!("获取dns记录成功:{:?}", dns_records);
                                                Message::QueryDnsResult(dns_records)
                                            },
                                        ),
                                        Task::perform(
                                            Self::handle_dns_operate_log_query(name_for_log_query),
                                            |dns_records| {
                                                println!("获取dns记录成功:{:?}", dns_records);
                                                Message::QueryDnsLogResult(dns_records)
                                            },
                                        ),
                                    ])
                                }
                                None => Task::none(),
                            }
                        }
                    }
                    _ => {
                        // 其他的页面切换事件不处理
                        Task::none()
                    }
                }
            }
            Message::QueryDnsResult(dns_list) => {
                self.dns_list = dns_list;
                Task::none()
            }
            Message::QueryDnsLogResult(logs) => {
                dbg!("dns操作日志查询成功");
                self.dns_log_list = logs;
                Task::none()
            }
            Message::DomainDeleted(domain_name) => {
                dbg!("删除域名：domain_name", domain_name);
                Task::none()
            }
            Message::AddDomainFormChanged(domain_name) => {
                self.add_domain_field.domain_name = domain_name;
                Task::none()
            }
            Message::SubmitDomainForm => {
                dbg!(
                    "提交域名表单：添加完毕",
                    &self.add_domain_field.domain_name,
                    &self.add_domain_field.provider
                );
                self.domain_names
                    .push(self.add_domain_field.domain_name.clone().into());
                self.update(Message::ChangePage(Page::DomainPage))
            }
            Message::QueryDomainDnsRecord(domain_name) => {
                dbg!("查询dns记录：domain_name:", &domain_name);
                self.current_domain_name = Some(domain_name.clone());
                self.update(Message::ChangePage(Page::DnsRecord))
            }
            Message::DnsProviderSelected(provider) => {
                dbg!("dns provider selected: ", provider);
                self.add_domain_field.provider = Some(provider);
                Task::none()
            }
            Message::ToHelp => self.update(Message::ChangePage(Page::Help)),
            Message::KeyInput { key } => {
                let msg = handle_key(&self, &key);
                match msg {
                    Some(msg) => self.update(msg),
                    None => Task::none(),
                }
            }
            Message::OpenHelp { last_page } => {
                self.last_page = Some(last_page);
                self.update(Message::ChangePage(Page::Help))
            }
            Message::CloseHelp => match &self.last_page {
                Some(page) => self.update(Message::ChangePage(page.clone())),
                _ => Task::none(),
            },
            Message::QueryDomain => {
                dbg!("查询域名");
                if !self.in_query {
                    self.in_query = true;
                    Task::perform(Self::handle_domain_reload(), |domain_names| {
                        println!("请求接口信息:{:?}", domain_names);
                        Message::QueryDomainResult(domain_names)
                    })
                } else {
                    info!("正在查询中，请勿重复点击！");
                    Task::none()
                }
            }
            Message::QueryDomainResult(domain_names) => {
                self.domain_names = domain_names;
                self.in_query = false;
                self.update(Message::ChangePage(Page::DomainPage))
            }
            Message::DnsDelete(record_id) => {
                dbg!("删除dns记录:{}", &record_id);
                Task::perform(Self::handle_dns_record_delete(record_id), |response| {
                    println!("请求接口信息:{:?}", response);
                    match response {
                        None => Message::ChangePage(Page::DnsRecord),
                        Some(record_id) => Message::DnsRecordDeleted(record_id.clone()),
                    }
                })
            }
            Message::AddDnsRecord => match &self.current_domain_name {
                Some(domain_name) => {
                    let name = domain_name.get_domain_name();
                    self.add_dns_form = AddDnsField {
                        domain_name: name.to_string(),
                        ..AddDnsField::default()
                    };
                    self.update(Message::ChangePage(Page::AddRecord))
                }
                None => Task::none(),
            },
            Message::DnsFormNameChanged(record_name) => {
                dbg!("添加dns记录表单变化：", &record_name);
                self.add_dns_form = AddDnsField {
                    record_name,
                    ..self.add_dns_form.clone()
                };
                Task::none()
            }
            Message::AddDnsFormSubmit => match self.add_dns_form.validate() {
                true => {
                    dbg!("添加dns记录表单提交：", &self.add_dns_form);
                    Task::perform(
                        Self::handle_dns_record_add(AddDnsField {
                            ..self.add_dns_form.clone()
                        }),
                        |domain_names| {
                            println!("请求接口信息:{:?}", domain_names);
                            Message::ChangePage(Page::AddRecord)
                        },
                    )
                }
                false => {
                    dbg!("添加dns记录表单提交失败：", &self.add_dns_form);
                    Task::none()
                }
            },
            Message::DnsFormRecordTypeChanged(record_type) => {
                dbg!("添加dns记录表单变化：", &record_type);
                self.handle_dns_add(AddDnsField {
                    record_type: Some(record_type),
                    ..self.add_dns_form.clone()
                });
                Task::none()
            }
            Message::DnsFormValueChanged(value) => {
                dbg!("添加dns记录表单变化：", &value);
                self.handle_dns_add(AddDnsField {
                    value,
                    ..self.add_dns_form.clone()
                });
                Task::none()
            }
            Message::DnsFormTtlChanged(ttl) => {
                dbg!("添加dns记录表单变化：", ttl);
                // 这里会不会卡呀
                self.handle_dns_add(AddDnsField {
                    ttl,
                    ..self.add_dns_form.clone()
                });
                Task::none()
            }
            Message::AddDnsFormCancelled => {
                // 提交表单恢复原状
                self.add_dns_form = AddDnsField::default();
                // 返回到dns管理界面
                self.update(Message::ChangePage(Page::DnsRecord))
            }
            Message::DnsRecordDeleted(record_id) => {
                self.dns_list.retain(|record| record.record_id != record_id);
                // 返回到dns管理界面
                self.update(Message::ChangePage(Page::DnsRecord))
            }
            Message::Quit => {
                process::exit(0);
            }

            _ => {
                // dbg!("未处理的消息：{:?}", message);
                Task::none()
            }
        }
    }

    async fn handle_domain_reload() -> Vec<DomainName> {
        dbg!("查询域名信息");
        let domain_list = query_aliyun_domain_list();
        domain_list
    }

    async fn handle_dns_reload(domain_name: String) -> Vec<Record> {
        dbg!("查询域名信息");
        let domain_list = query_aliyun_dns_list(domain_name);
        domain_list
    }

    async fn handle_dns_operate_log_query(domain_name: String) -> Vec<RecordLog> {
        dbg!("查询域名信息");
        let dns_operate_logs = query_aliyun_dns_operation_list(domain_name);
        dns_operate_logs
    }

    async fn handle_dns_record_add(domain_name: AddDnsField) -> bool {
        dbg!("添加域名解析记录");
        add_aliyun_dns_record(&domain_name)
    }

    async fn handle_dns_record_delete(record_id: String) -> Option<String> {
        dbg!("删除域名解析记录");
        delete_aliyun_dns(record_id)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn handle_dns_add(&mut self, form: AddDnsField) {
        self.add_dns_form = form;
    }

    // 监听键盘
    fn keyboard_subscription(_: &DomainManager) -> Subscription<Message> {
        dbg!("创建键盘监听");
        let key = keyboard::on_key_press(|key, _| {
            dbg!("监听到键盘事件：{:?}", &key);
            let msg = Message::KeyInput { key };
            Some(msg)
        });
        Subscription::batch([key])
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([DomainManager::window_subscription()])
    }

    fn window_subscription() -> Subscription<Message> {
        iced::event::listen_with(|event, _, _| match event {
            Window(window::Event::Focused) => Some(Message::WindowFocused),
            Window(window::Event::Moved(Point { x, y })) => Some(Message::WindowMoved(x, y)),
            Window(window::Event::Resized(Size { width, height })) => {
                Some(Message::WindowResized(width, height))
            }
            Window(window::Event::CloseRequested) => Some(Message::QuitWrapper),
            _ => None,
        })
    }
}

///
/// 处理按键事件
fn handle_key(app: &DomainManager, key: &Key) -> Option<Message> {
    // 在其他所有界面，如果按下h，进入帮助界面
    // 在帮助界面，如果按下h，退出帮助界面
    match app.current_page {
        Page::Help => {
            if let Key::Character(c) = key {
                match c.as_str() {
                    "h" => Some(Message::CloseHelp),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => {
            if let Key::Character(c) = key {
                info!("在{}页面按下{}键", app.current_page, c.as_str());
                match c.as_str().to_lowercase().as_str() {
                    "h" => {
                        info!("监听到按下h键,关闭帮助界面");
                        Some(Message::OpenHelp {
                            last_page: app.current_page.clone(),
                        })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::i18_utils::get_text;
    use crate::{Config, DomainManager, Message};
    use serial_test::parallel;

    #[test]
    fn test_get_text() {
        rust_i18n::set_locale("en");
        assert_eq!("Hello World!", get_text("hello"));
        rust_i18n::set_locale("zh_CN");
        assert_eq!("你好世界！", get_text("hello"));
        assert_eq!("返回", get_text("return"));
    }

    #[test]
    fn test_parse_json_config() {
        let config = Config::new_from_file("config.json");
        assert_eq!(config.name, "Domain Manager");
    }

    #[test]
    #[parallel] // needed to not collide with other tests generating configs files
    fn test_correctly_update_ip_version() {
        let mut app = DomainManager::default();
        let _ = app.update(Message::AddDnsRecord);
    }
}
