mod ali_api;
mod locale;
mod model;
mod view;

use iced::alignment::Horizontal;

use crate::ali_api::{
    add_aliyun_dns_record, delete_aliyun_dns, query_aliyun_dns_list,
    query_aliyun_dns_operation_list, query_aliyun_domain_list,
};
use crate::locale::Locale;
use crate::model::dns_operate::RecordLog;
use crate::model::dns_record_response::Type;
use crate::view::domain::{add_domain_page, domain_page};
use crate::view::domain_dns_record::{add_dns_record, dns_record};
use iced::keyboard::Key;
use iced::widget::{button, column, row, text, Column, Container};
use iced::window::Position;
use iced::{
    application, keyboard, window, Element, Font, Length, Settings, Size, Subscription, Task, Theme,
};
use log::{error, info};
use model::dns_record_response::Record;
use rust_i18n::{i18n, t};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::sync::OnceLock;

const TITLE_SIZE: u16 = 36;
const TITLE_PADDING: u16 = 20;
const CONTENT_SIZE: u16 = 20;

const DOMAIN_MANAGER_LOWERCASE: &str = "domain_manager";

i18n!("locales", fallback = "en");

#[derive(Serialize, Deserialize, Debug)]
enum LICENCE {
    MIT,
    Apache,
    MulanPSL2,
}

///
///   "name": "Domain Manager",
//     "description": "A simple domain manager",
//     "version": "1.0.0",
//     "author": "Stanic.xyz",
//     "license": "Mulan PSL v2"
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    // ...
    domain_names: Vec<DomainName>,
    name: String,
    description: String,
    version: String,
    author: String,
    license: LICENCE,
    locale: String,
}

impl From<String> for Config {
    fn from(value: String) -> Self {
        Config {
            domain_names: vec![],
            name: value,
            description: String::new(),
            version: String::new(),
            author: String::new(),
            license: LICENCE::MulanPSL2,
            locale: String::from("zh-CN"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            domain_names: vec![],
            name: String::from(DOMAIN_MANAGER_LOWERCASE),
            description: String::new(),
            version: String::new(),
            author: String::new(),
            license: LICENCE::MulanPSL2,
            locale: String::from("en"),
        }
    }
}

impl Config {
    /// 加载配置文件
    ///
    pub fn new_from_string(file_content: &String) -> Self {
        dbg!("文件内容", file_content);
        let config: Config = serde_json::from_str(&file_content).unwrap();
        config.into()
    }

    /// 加载配置文件
    ///
    pub fn new_from_file(file_name: &str) -> Self {
        let file = Self::load_file(&file_name);
        if let Some(ref file_content) = file {
            dbg!("Loading file content: {}", &file_content);
            Self::new_from_string(&file_content).into()
        } else {
            error!("Loading config file failed!");
            Self::default()
        }
    }

    ///
    /// Load a file from the static directory
    ///
    fn load_file(file_name: &str) -> Option<String> {
        println!("load_file: {}", file_name);
        let default_path = format!("{}\\config\\", env!("CARGO_MANIFEST_DIR"));
        println!("default_path: {}", default_path);
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let file_path = format!("{}{}", public_path, file_name);
        println!("Loading file: {}", file_path);
        read_to_string(file_path).ok()
    }
}

pub fn main() -> iced::Result {
    // 读取配置文件
    let config = Config::new_from_file("config.json");
    info!(
        "配置文件信息：应用名称：{}，语言：{}",
        &config.name, &config.locale
    );
    let app = App::new(config);
    app.start()
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    ChangeLocale,
    ToggleTheme,
    ChangePage(Page),
    PageChanged(Page, Page),
    SubmitDomainForm,
    DomainDeleted(DomainName),
    AddDomainFormChanged(String),
    DnsProviderSelected(DnsProvider),
    QueryDomainDnsRecord(DomainName),
    ToHelp,
    KeyInput { key: Key },
    CloseHelp,
    OpenHelp { last_page: Page },
    QueryDomain,
    QueryDomainResult(Vec<DomainName>),
    QueryDnsResult(Vec<Record>),
    QueryDnsLogResult(Vec<RecordLog>),
    DnsDelete(String),
    AddDnsRecord,
    DnsFormNameChanged(String),
    DnsFormRecordTypeChanged(Type),
    DnsFormValueChanged(String),
    DnsFormTtlChanged(i32),
    AddDnsFormSubmit,
    AddDnsFormCancelled,
    DnsRecordDeleted(String),
}

#[derive(Debug, Clone)]
struct AddDomainField {
    domain_name: String,
    provider: Option<DnsProvider>,
}

impl Default for AddDomainField {
    fn default() -> Self {
        AddDomainField {
            domain_name: String::new(),
            provider: Some(DnsProvider::Aliyun),
        }
    }
}

#[derive(Debug, Clone)]
struct AddDnsField {
    record_id: Option<String>,
    domain_name: String,
    record_name: String,
    value: String,
    ttl: i32,
    record_type: Option<Type>,
}

impl AddDnsField {
    /// update_input
    pub fn update_value(&mut self) {
        self.record_name = String::new();
    }

    pub(crate) fn validate(&self) -> bool {
        println!("验证输入是否合法！{:?}", &self);
        true
    }
}

impl Default for AddDnsField {
    fn default() -> Self {
        AddDnsField {
            record_id: None,
            record_name: String::new(),
            domain_name: String::new(),
            ttl: 600,
            record_type: Some(Type::A),
            value: String::new(),
        }
    }
}

#[derive(Debug)]
struct App {
    config: Config,
    theme: Theme,
    domain_names: Vec<DomainName>,
    current_page: Page,
    current_domain_name: Option<DomainName>,
    add_domain_field: AddDomainField,
    last_page: Option<Page>,
    // 当前处于查询状态
    in_query: bool,
    dns_list: Vec<Record>,        // 当前域名对应的DNS记录
    dns_log_list: Vec<RecordLog>, // 当前域名对应的DNS记录
    add_dns_form: AddDnsField,
    locale: Locale,
}

// 监听键盘
fn subscribe(_: &App) -> Subscription<Message> {
    let key = keyboard::on_key_press(|key, _| {
        let msg = Message::KeyInput { key };
        Some(msg)
    });
    Subscription::batch([key])
}

impl Default for App {
    fn default() -> Self {
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
            config: Config {
                name: String::from("Domain Manager"),
                description: String::from("A simple domain manager"),
                version: String::from("1.0.0"),
                author: String::from("Stanic.xyz"),
                license: LICENCE::MulanPSL2,
                domain_names: vec![],
                locale: String::from("en"),
            },
        }
    }
}

// 定义主题
impl App {
    pub fn start(&self) -> iced::Result {
        self.locale();
        application("Dns Manager", Self::update, Self::view)
            .subscription(subscribe)
            .window(window::Settings {
                size: Size::new(1080f32, 720f32), // start size
                position: Position::Default,
                min_size: Some(Size::new(1080f32, 720f32)), // Some(ConfigWindow::MIN_SIZE.to_size()), // min size allowed
                max_size: None,
                visible: true,
                resizable: true,
                decorations: true,
                transparent: true,
                exit_on_close_request: true,
                ..Default::default()
            })
            .settings(Settings {
                fonts: vec![include_bytes!("../fonts/MapleMono-NF-CN-Regular.ttf").into()],
                default_font: Font::with_name("Maple Mono NF CN"),
                ..Default::default()
            })
            .theme(Self::theme)
            .run()
    }

    fn locale(&self) {
        match self.locale {
            Locale::Chinese => rust_i18n::set_locale("zh_CN"),
            Locale::English => rust_i18n::set_locale("en"),
        }
    }

    fn new(config: Config) -> Self {
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
            dns_list: vec![],
            dns_log_list: vec![],
            add_dns_form: AddDnsField::default(),
            locale,
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.current_page {
            Page::DomainPage => domain_page(&self),
            // 添加域名界面
            Page::AddDomain => add_domain_page(&self),
            Page::DnsRecord => dns_record(&self),
            Page::AddRecord => add_dns_record(&self),
            Page::Help => help(&self),
        }
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
                if self.theme == Theme::Light {
                    self.theme = Theme::Dark
                } else {
                    self.theme = Theme::Light
                }
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
                                    let name: String = domain_name.name.clone();
                                    let name_for_log_query: String = domain_name.name.clone();

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
                    let name = domain_name.name.clone();
                    self.add_dns_form = AddDnsField {
                        domain_name: name,
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
}

///
/// 处理按键事件
fn handle_key(app: &App, key: &Key) -> Option<Message> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
enum DnsProvider {
    Aliyun,
    TencentCloud,
    CloudFlare,
    Tomato,
}

impl DnsProvider {
    const ALL: [DnsProvider; 4] = [
        DnsProvider::Aliyun,
        DnsProvider::TencentCloud,
        DnsProvider::CloudFlare,
        DnsProvider::Tomato,
    ];
}

impl Display for DnsProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsProvider::Aliyun => write!(f, "Aliyun"),
            DnsProvider::TencentCloud => write!(f, "TencentCloud"),
            DnsProvider::CloudFlare => write!(f, "CloudFlare"),
            DnsProvider::Tomato => write!(f, "Tomato"),
        }
    }
}

fn help(_app: &App) -> Element<Message> {
    let title = button(
        text(get_text("help.title"))
            .width(Length::Shrink)
            .size(TITLE_SIZE)
            .align_x(Horizontal::Center),
    )
    .on_press(Message::ChangePage(Page::DomainPage));

    let title = row!(title).padding(TITLE_PADDING);

    let mut content = Column::new().padding(5);
    for (key, desc) in get_help_text() {
        let to_text = |s| text(s).width(Length::Shrink).size(CONTENT_SIZE);
        let (key, desc) = (to_text(key), to_text(desc));
        let row = row!(key, desc).spacing(50).padding(2);
        content = content.push(row);
    }
    let content = Container::new(content)
        .width(Length::Fill)
        .center_x(Length::Fill);

    let container = column!(title, content).spacing(20);
    let container = Container::new(container)
        .width(Length::Shrink)
        .center_x(Length::Fill);

    container.into()
}

const KEY_DESCRIPTION: &[(&str, &str)] = &[
    ("\n● 模式/播放", "\n"),
    ("h", "进入帮助页面"),
    ("[p, space]", "播放/暂停"),
    ("t", "切换语言(默认双语字幕, 每次切换至中文/日语/双语)"),
    ("s", "切换播放速度"),
    ("q", "关闭应用"),
    ("\n\n● 模式/帮助", "\n"),
    ("h", "退出帮助页面"),
    ("\n\n● 模式/退出", "\n"),
    ("y", "确认"),
    ("n", "取消"),
];

fn get_help_text() -> &'static Vec<(String, String)> {
    static KEY_DESCRIPTION_CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();

    KEY_DESCRIPTION_CACHE.get_or_init(|| {
        let get_len = |s: &str| {
            s.chars()
                .fold(0, |acc, ch| acc + if ch.is_ascii() { 1 } else { 2 })
        };

        let get_format = |s: &str, max_len: usize| {
            let count = max_len - get_len(s);
            String::from(s) + " ".repeat(count).as_str()
        };

        let (mut key_max_len, mut desc_max_len) = (0, 0);
        for (key, desc) in KEY_DESCRIPTION {
            key_max_len = get_len(key).max(key_max_len);
            desc_max_len = get_len(desc).max(desc_max_len);
        }

        KEY_DESCRIPTION
            .iter()
            .map(|(key, desc)| {
                let key = get_format(key, key_max_len);
                let desc = get_format(desc, desc_max_len);
                (key, desc)
            })
            .collect()
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct DomainName {
    provider: DnsProvider,
    name: String, // e.g. example.com ,
    dns_record: Vec<DnsRecord>,
}

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        DomainName {
            name: value,
            provider: DnsProvider::Tomato,
            dns_record: vec![],
        }
    }
}

impl Display for DomainName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Default for DomainName {
    fn default() -> Self {
        DomainName {
            name: String::from(""),
            provider: DnsProvider::Tomato,
            dns_record: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    DomainPage,
    AddDomain,
    DnsRecord,
    AddRecord,
    Help,
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::DomainPage => write!(f, "page_domain_manage"),
            Page::AddDomain => write!(f, "page_add_domain"),
            Page::DnsRecord => write!(f, "DnsRecord"),
            Page::AddRecord => write!(f, "{}", get_text("add_record")),
            Page::Help => write!(f, "Help"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
struct DnsRecord {
    domain_name: String,
    dns_name: String,
    dns_type: String,
    dns_value: String,
    ttl: i64,
}

fn get_text(name: &str) -> String {
    t!(name).into()
}

#[cfg(test)]
mod tests {
    use crate::{get_text, App, Config, Message};
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
        let mut app = App::default();
        let _ = app.update(Message::AddDnsRecord);
    }
}
