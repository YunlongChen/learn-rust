mod ali_api;
mod dns_record_response;
mod domain;

use iced::alignment::Horizontal;

use crate::ali_api::{query_aliyun_dns_list, query_aliyun_domain_list};
use crate::dns_record_response::Record;
use iced::border::Radius;
use iced::keyboard::Key;
use iced::widget::button::{danger, primary};
use iced::widget::text::LineHeight;
use iced::widget::{
    button, column, container, pick_list, row, text, text_input, Button, Column, Container, Row,
    Text, TextInput,
};
use iced::window::Position;
use iced::{
    application, color, keyboard, window, Alignment, Background, Border, Color, Element, Font,
    Length, Padding, Settings, Size, Subscription, Task, Theme,
};
use log::{error, info};
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
    rust_i18n::set_locale(config.locale.as_str());
    let app = App::new(config);
    app.start()
}

#[derive(Debug, Clone)]
enum Message {
    // ...
    Increment,
    Decrement,
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
    DnsDelete,
    DnsEdit(Record),
    AddDnsRecord,
    DnsFormContentChanged(String),
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
}

impl Default for AddDnsField {
    fn default() -> Self {
        AddDnsField {
            record_id: None,
            domain_name: String::new(),
        }
    }
}

#[derive(Debug)]
struct App {
    config: Config,
    theme: Theme,
    domain_names: Vec<DomainName>,
    counter: i32,
    current_page: Page,
    current_domain_name: Option<DomainName>,
    add_domain_field: AddDomainField,
    last_page: Option<Page>,
    // 当前处于查询状态
    in_query: bool,
    dns_list: Vec<Record>, // 当前域名对应的DNS记录
    add_dns_form: AddDnsField,
}

impl App {
    pub fn start(&self) -> iced::Result {
        application("Dns manager", Self::update, Self::view)
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
            counter: 0,
            theme: Theme::Dark,
            domain_names: vec![],
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            add_dns_form: AddDnsField::default(),
            last_page: None,
            in_query: false,
            dns_list: vec![],
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
    fn new(config: Config) -> Self {
        // 初始化数据
        let domain_names = config.domain_names.clone();
        Self {
            current_page: Page::DomainPage,
            counter: 0,
            theme: Theme::Light,
            domain_names,
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            last_page: None,
            in_query: false,
            config,
            dns_list: vec![],
            add_dns_form: AddDnsField::default(),
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.current_page {
            Page::DomainPage => domain_page(&self),
            // 添加域名界面
            Page::AddDomain => add_domain_page(&self),
            Page::DnsRecord => {
                // 展示dns列表
                match &self.current_domain_name {
                    // 选中了域名
                    Some(domain_name) => {
                        // 返回到解析界面
                        let dns_content: Column<Message> =
                            Column::from_iter(self.dns_list.iter().map(|record: &Record| {
                                // 这里是一行数据
                                row![
                                    text!("{}", record.rr).width(Length::Fixed(200.0)),
                                    text!("{}", record.record_type)
                                        .width(Length::Fixed(100.0))
                                        .align_x(Alignment::Start),
                                    text!("{}", record.value)
                                        .width(Length::Fill)
                                        .line_height(LineHeight::default())
                                        .style(|_theme: &Theme| { text::Style::default() })
                                        .align_x(Alignment::Start),
                                    button(Text::new(get_text("edit")).align_x(Alignment::Center))
                                        .style(|theme: &Theme, status| {
                                            match status {
                                                button::Status::Hovered => button::Style::default()
                                                    .with_background(Color::from_rgb(
                                                        255.0, 50.0, 50.0,
                                                    )),
                                                _ => primary(theme, status),
                                            }
                                        })
                                        .on_press(Message::DnsEdit(record.clone()))
                                        .width(Length::Fixed(100.0)),
                                    button(
                                        Text::new(get_text("delete")).align_x(Alignment::Center)
                                    )
                                    .style(|theme: &Theme, status| {
                                        match status {
                                            button::Status::Hovered => button::Style::default()
                                                .with_background(Color::from_rgb(
                                                    255.0, 50.0, 50.0,
                                                )),
                                            _ => danger(theme, status),
                                        }
                                    })
                                    .on_press(Message::DnsDelete)
                                    .width(Length::Fixed(100.0))
                                ]
                                .align_y(Alignment::Center)
                                .into()
                            }));

                        let title: String = match self.in_query {
                            true => format!(
                                "{}：{}({})",
                                get_text("dns_record"),
                                domain_name.name,
                                get_text("in_query")
                            ),
                            false => format!("{}：{}", get_text("dns_record"), domain_name.name),
                        };

                        Column::new()
                            .push(
                                row![
                                    button(Text::new(get_text("return")).center())
                                        .on_press(Message::ChangePage(Page::DomainPage)),
                                    row!(
                                        text!("{}", title).width(Length::Fill).center(),
                                    ).width(Length::Fill)
                                    ,
                                    button(Text::new("Help").center())
                                        .on_press(Message::ToHelp)
                                        .width(Length::Fixed(100.0)),
                                    button(Text::new(get_text("reload")))
                                        .on_press(Message::ToHelp)
                                        .width(Length::Fixed(100.0)),
                                    button(Text::new(get_text("add_dns_record")).center())
                                        .on_press(Message::AddDnsRecord)
                                        .width(Length::Fixed(200.0))
                                ]
                                    .padding(Padding {
                                        bottom: 20.0,
                                        ..Default::default()
                                    })
                                    .align_y(Alignment::Center),
                            )
                            .push(
                                // 选中了域名
                                text!("Dns Record list for domain：{}", domain_name.name)
                                    .width(Length::Fill),
                            )
                            .push_maybe(match self.in_query {
                                true => Some(text!("{}", get_text("in_query")).width(Length::Fill)),
                                false => None,
                            })
                            // dns 列表
                            .push(
                                row![
                                    text!("主机记录").width(Length::Fixed(200.0)),
                                    text!("记录类型")
                                        .width(Length::Fixed(100.0))
                                        .align_x(Alignment::Start),
                                    text!("记录值")
                                        .width(Length::Fill)
                                        .line_height(LineHeight::default())
                                        .style(|_theme: &Theme| { text::Style::default() })
                                        .align_x(Alignment::Start),
                                    text("操作")
                                        .align_x(Alignment::Center)
                                        .width(Length::Fixed(200.0))
                                ]
                                    .align_y(Alignment::Center),
                            )
                            .push(dns_content)
                            .push(Container::new(row![
                                text!("Dns解析操作记录").width(Length::Fill),
                                button(Text::new(get_text("reload"))).width(Length::Fixed(100.0))
                                .on_press(Message::ToHelp)
                            ])
                                .padding(Padding {
                                    bottom: 20.0,
                                    ..Default::default()
                                })
                                .style(|_theme: &Theme| {
                                    // 北京颜色
                                    container::Style {
                                        text_color: Some(Color::WHITE),
                                        border: Border{
                                            color: Color::from_rgb(255.0, 100.2, 0.0),
                                            radius: Radius::from(5),
                                            ..Default::default()
                                        },
                                        ..container::Style::default()
                                    }
                                })
                                .width(Length::Fill)
                                .align_y(Alignment::Center)
                                .align_x(Alignment::Start)
                            )
                            .push(row![
                                text!("操作时间")
                                    .width(Length::Fixed(200.0))
                                    .align_x(Alignment::Start),
                                text("操作方式")
                                    .width(Length::Fixed(100.0))
                                    .line_height(LineHeight::default()),
                                text("详细信息")
                                    .width(Length::Fill)
                                    .line_height(LineHeight::default())
                            ])
                            .push(row![
                                text!("2024-12-20T21:44Z")
                                    .width(Length::Fixed(200.0))
                                    .align_x(Alignment::Start),
                                text("ADD")
                                    .width(Length::Fixed(100.0))
                                    .line_height(LineHeight::default()),
                                text("Add resolution record. A record fnos Default 192.168.9.103 ( TTL: 600)")
                                    .width(Length::Fill)
                                    .line_height(LineHeight::default())
                            ])
                            .padding(10)
                            .spacing(10)
                            .into()
                    }
                    None => {
                        // 没有选择域名，返回到域名列表(这里除非是除了BUG，应该不会走到这里来）
                        text!("No Domain Name selected!").width(Length::Fill).into()
                    }
                }
            }
            Page::AddRecord => {
                let record_id_column = match &self.add_dns_form.record_id {
                    Some(record_id) => text!("修改Dns记录：{}", record_id)
                        .width(Length::Fill)
                        .into(),
                    None => text!("No record id").width(Length::Fill).into(),
                };

                // 添加 dns 记录
                Container::new(
                    Column::new()
                        .width(Length::Fill)
                        .align_x(Alignment::Start)
                        .push_maybe(record_id_column)
                        .push(
                            Text::new(get_text("add_dns_record"))
                                .width(Length::Fill)
                                .center(),
                        )
                        .push(
                            Column::new()
                                .push(text!("记录类型").width(Length::Fill))
                                .push(
                                    text_input(
                                        "Type something here...",
                                        &self.add_dns_form.domain_name,
                                    )
                                    .on_input(Message::DnsFormContentChanged),
                                )
                                .push(text!("主机记录").width(Length::Fill))
                                .push(
                                    text_input(
                                        "Type something here...",
                                        &self.add_dns_form.domain_name,
                                    )
                                    .on_input(Message::DnsFormContentChanged),
                                )
                                .push(text!("请求来源").width(Length::Fill))
                                .push(
                                    text_input(
                                        "Type something here...",
                                        &self.add_dns_form.domain_name,
                                    )
                                    .on_input(Message::DnsFormContentChanged),
                                )
                                .push(text!("记录值").width(Length::Fill))
                                .push(
                                    text_input(
                                        "Type something here...",
                                        &self.add_dns_form.domain_name,
                                    )
                                    .on_input(Message::DnsFormContentChanged),
                                )
                                .push(text!("TTL").width(Length::Fill))
                                .push(
                                    text_input(
                                        "Type something here...",
                                        &self.add_dns_form.domain_name,
                                    )
                                    .on_input(Message::DnsFormContentChanged),
                                )
                                .width(Length::Fill),
                        )
                        .push(
                            Row::new()
                                .push(
                                    button(Text::new(get_text("cancel")))
                                        .on_press(Message::ChangePage(Page::DnsRecord))
                                        .width(Length::Fixed(200.0)),
                                )
                                .push(
                                    button(Text::new(get_text("confirm")))
                                        .on_press(Message::AddDnsRecord)
                                        .width(Length::Fixed(200.0)),
                                )
                                .spacing(20)
                                .width(Length::Fill)
                                .align_y(Alignment::Center),
                        ),
                )
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
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
            Message::Increment => {
                self.counter += 1;
                Task::none()
            }
            Message::Decrement => {
                if self.counter > 0 {
                    self.counter -= 1
                }
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
                                    Task::perform(Self::handle_dns_reload(name), |dns_records| {
                                        println!("获取dns记录成功:{:?}", dns_records);
                                        Message::QueryDnsResult(dns_records)
                                    })
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
            Message::DnsDelete => {
                dbg!("删除dns记录");
                Task::none()
            }
            Message::AddDnsRecord => self.update(Message::ChangePage(Page::AddRecord)),
            Message::DnsFormContentChanged(record) => {
                dbg!("添加dns记录表单变化：", record);
                Task::none()
            }
            Message::DnsEdit(record) => {
                self.add_dns_form = AddDnsField {
                    record_id: Some(record.record_id), // 记录ID
                    domain_name: record.value,
                    ..Default::default()
                };
                self.update(Message::ChangePage(Page::AddRecord))
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

    fn theme(&self) -> Theme {
        self.theme.clone()
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

fn add_domain_page(app: &App) -> Element<'static, Message> {
    let state = AddDomainField {
        domain_name: String::from("www.example.com"),
        provider: app.add_domain_field.provider.clone(),
    };

    Container::new(column![
        text("add domain")
            .color(color!(0x0000ff))
            .size(20)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(color!(0xff00ff)),
                }
            })
            .width(Length::Fill),
        TextInput::new("domain name", &app.add_domain_field.domain_name)
            .on_input(Message::AddDomainFormChanged),
        pick_list(
            &DnsProvider::ALL[..],
            state.provider,
            Message::DnsProviderSelected
        )
        .placeholder("Select your favorite fruit..."),
        Button::new(text("confirm")).on_press(Message::SubmitDomainForm),
        button(Text::new(get_text("return"))).on_press(Message::ChangePage(Page::DomainPage)),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .align_top(0)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

// 夏青.我爱你
/// 域名管理界面
fn domain_page(app: &App) -> Element<'static, Message> {
    // 返回到解析界面
    let domain_name_list: Column<Message> =
        Column::from_iter(app.domain_names.iter().map(|domain_name: &DomainName| {
            // 这里是一行数据
            let counter: i32 = app.counter;
            row![
                text!("{counter}").width(Length::Fill),
                text!("{}", domain_name.name).width(Length::Fill),
                text!("{}", domain_name.provider)
                    .width(Length::Fill)
                    .line_height(LineHeight::default())
                    .style(|_theme: &Theme| { text::Style::default() })
                    .align_x(Alignment::Start),
                button(Text::new(t!("dns_record")))
                    .on_press(Message::QueryDomainDnsRecord(domain_name.clone()))
                    .width(Length::Fill)
            ]
            .align_y(Alignment::Center)
            .into()
        }));

    let in_query_tag = if app.in_query {
        Some(Text::new(get_text("in_query")).width(Length::Fill))
    } else {
        None
    };

    let actions = Row::new()
        .push_maybe(in_query_tag)
        .push(
            button(text(get_text("reload")).align_x(Alignment::Center))
                .on_press(Message::QueryDomain)
                .width(Length::Fill),
        )
        .push(
            button(Text::new(get_text("add_domain")).align_x(Alignment::Center))
                .on_press(Message::ChangePage(Page::AddDomain))
                .width(Length::Fill),
        )
        .spacing(10)
        .width(Length::Fill)
        .align_y(Alignment::End);

    let row1 = Column::new().push(actions).push(
        Container::new(domain_name_list)
            .style(container::rounded_box)
            .width(Length::Fill)
            .height(Length::Fill),
    );

    let content2: Column<Message> = Column::new().push(row1).width(Length::Fill);

    Container::new(
        content2.push(text!("Made with Love by {}", app.config.author).align_x(Alignment::End)),
    )
    .style(container::rounded_box)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .center(800)
    .center_x(Length::Fill)
    .center_y(Length::Shrink)
    .into()
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
// DNS 解析类型
#[derive(Debug, Clone)]
enum DomainType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    SOA,
    PTR,
    SRV,
    // 添加其他域名解析类型
}

impl Display for DomainType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DomainType::A => write!(f, "A"),
            DomainType::AAAA => write!(f, "AAAA"),
            DomainType::CNAME => write!(f, "CNAME"),
            DomainType::MX => write!(f, "MX"),
            DomainType::TXT => write!(f, "TXT"),
            DomainType::NS => write!(f, "NS"),
            DomainType::SOA => write!(f, "SOA"),
            DomainType::PTR => write!(f, "PTR"),
            DomainType::SRV => write!(f, "SRV"),
            // 处理其他域名解析类型
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
struct DnsRecord {
    domain_name: String,
    dns_name: String,
    dns_type: String, // A, AAAA, CNAME, MX, TXT, etc.
    dns_value: String,
    ttl: i64,
}

fn get_text(name: &str) -> String {
    t!(name).into()
}

#[test]
fn it_counts_properly() {
    let mut app = App::default();

    let _ = app.update(Message::Increment);
    let _ = app.update(Message::Increment);
    let _ = app.update(Message::Decrement);

    assert_eq!(app.counter, 1);
}

#[cfg(test)]
mod tests {
    use crate::{get_text, Config};

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
}
