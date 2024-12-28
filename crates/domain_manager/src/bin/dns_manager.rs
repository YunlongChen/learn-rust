use iced::alignment::Horizontal;
use iced::border::left;
use iced::keyboard::{Key, Modifiers};
use iced::widget::shader::wgpu::naga::MathFunction::Outer;
use iced::widget::text::LineHeight;
use iced::widget::{
    button, column, container, pick_list, row, stack, text, Button, Column, Container, Row, Text,
    TextInput,
};
use iced::window::settings::PlatformSpecific;
use iced::window::Position;
use iced::ContentFit::Contain;
use iced::{
    application, color, executor, keyboard, window, Alignment, Application, Color, Element, Font,
    Length, Settings, Size, Subscription, Theme,
};
use log::{error, info};
use rust_i18n::{i18n, t};
use serde::de::Unexpected::Str;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;
use winit::application::ApplicationHandler;

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
    SubmitDomainForm,
    DomainDeleted(DomainName),
    AddDomainFormChanged(String),
    DnsProviderSelected(DnsProvider),
    ToHelp,
    KeyInput { key: Key, modifiers: Modifiers },
    CloseHelp,
    OpenHelp { last_page: Page },
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

// # }

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
}

impl App {
    pub fn start(&self) -> iced::Result {
        iced::application("Dns manager", Self::update, Self::view)
            .subscription(subscribe)
            .window(window::Settings {
                size: Size::new(720f32, 320f32), // start size
                position: Position::Default,
                min_size: None, // Some(ConfigWindow::MIN_SIZE.to_size()), // min size allowed
                max_size: None,
                visible: true,
                resizable: true,
                decorations: true,
                transparent: true,
                #[cfg(target_os = "linux")]
                platform_specific: PlatformSpecific {
                    application_id: String::from(DOMAIN_MANAGER_LOWERCASE),
                    ..PlatformSpecific::default()
                },
                exit_on_close_request: true,
                ..Default::default()
            })
            .settings(Settings {
                fonts: vec![include_bytes!("MapleMono-NF-CN-Regular.ttf").into()],
                default_font: Font::with_name("Maple Mono NF CN"),
                ..Default::default()
            })
            .run()
    }
}

fn subscribe(state: &App) -> Subscription<Message> {
    let key = keyboard::on_key_press(|key, modifiers| {
        let msg = Message::KeyInput { key, modifiers };
        Some(msg)
    });
    Subscription::batch([key])
}

impl Default for App {
    fn default() -> Self {
        // 初始化数据
        let domain_names: Vec<DomainName> = vec![
            String::from("chenyunlong.cn").into(),
            String::from("stanic.xyz").into(),
        ];
        Self {
            current_page: Page::DomainPage,
            counter: 0,
            theme: Theme::Light,
            domain_names,
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            last_page: None,
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
        let domain_names: Vec<DomainName> = vec![
            String::from("chenyunlong.cn").into(),
            String::from("stanic.xyz").into(),
        ];
        Self {
            current_page: Page::DomainPage,
            counter: 0,
            theme: Theme::Light,
            domain_names,
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            last_page: None,
            config,
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.current_page {
            Page::DomainPage => domain_page(&self),
            // 添加域名界面
            Page::AddDomain => add_domain_page(&self),
            Page::DnsRecord(domain_name) => {
                dbg!("current domain!", domain_name);
                // 展示dns列表
                match &self.current_domain_name {
                    Some(domain_name) => {
                        // 选中了域名
                        text!("Dns Record list{}", domain_name.name)
                            .width(Length::Fill)
                            .into()
                    }
                    None => {
                        // 没有选择域名，返回到域名列表
                        text!("No Domain Name selected!").width(Length::Fill).into()
                    }
                }
            }
            Page::AddRecord => text!("add Dns Record").width(Length::Fill).into(),
            Page::Help => help(&self),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.counter += 1,
            Message::Decrement => {
                if self.counter > 0 {
                    self.counter -= 1
                }
            }
            Message::ToggleTheme => {
                if self.theme == Theme::Light {
                    self.theme = Theme::Dark
                } else {
                    self.theme = Theme::Light
                }
            }
            // 改变当前页面
            Message::ChangePage(page) => {
                info!("Page Changed");
                self.current_page = page;
            }
            Message::DomainDeleted(domain_name) => {
                dbg!("删除域名：domain_name", domain_name);
            }
            Message::AddDomainFormChanged(domain_name) => {
                self.add_domain_field.domain_name = domain_name
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
            Message::DnsProviderSelected(provider) => {
                dbg!("dns provider selected: ", provider);
                self.add_domain_field.provider = provider.into();
            }
            Message::ToHelp => self.update(Message::ChangePage(Page::Help)),
            Message::KeyInput {
                key,
                modifiers: Modifiers,
            } => {
                let msg = handle_key(&self, &key);
                match msg {
                    Some(msg) => self.update(msg),
                    None => {}
                }
            }
            Message::OpenHelp { last_page } => {
                self.last_page = Some(last_page);
                self.update(Message::ChangePage(Page::Help))
            }
            Message::CloseHelp => {
                match &self.last_page {
                    Some(page) => self.update(Message::ChangePage(page.clone())),
                    _ => {}
                }
            },
        }
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
                    "h" => {
                        Some(Message::CloseHelp)
                    },
                    _ => None,
                }
            }else {
                None   
            }
        }
        _ => {
            println!("其他页面，直接打开Help界面：{}", app.current_page);
            if let Key::Character(c) = key {
                match c.as_str() {
                    "h" => Some(Message::OpenHelp {
                        last_page:app.current_page.clone()
                    }),
                    _ => None,
                }
            } else {
                None   
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum DnsProvider {
    Aliyun,
    TencentCloud,
    CloudFlare,
    Tomato,
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
    let dns_providers = [
        DnsProvider::Aliyun,
        DnsProvider::TencentCloud,
        DnsProvider::CloudFlare,
        DnsProvider::Tomato,
    ];

    let state = AddDomainField {
        domain_name: String::from("www.example.com"),
        provider: app.add_domain_field.provider,
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
        pick_list(dns_providers, state.provider, Message::DnsProviderSelected,)
            .placeholder("Select your favorite fruit..."),
        Button::new(text("confirm")).on_press(Message::SubmitDomainForm),
        button("return to index!").on_press(Message::ChangePage(Page::DomainPage)),
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
                text!("Counter{counter}").width(Length::Fill),
                text!("{}", domain_name.name).width(Length::Fill),
                text!("{}", domain_name.provider)
                    .width(Length::Fill)
                    .line_height(LineHeight::default())
                    .style(|_theme: &Theme| { text::Style::default() })
                    .align_x(Alignment::Start),
                button("Add Dns")
                    .on_press(Message::DomainDeleted(domain_name.clone()))
                    .width(Length::Fill),
                button(Text::new(t!("dns_record")))
                    .on_press(Message::ChangePage(Page::DnsRecord(domain_name.clone())))
                    .width(Length::Fill),
                button(Text::new("Help"))
                    .on_press(Message::ToHelp)
                    .width(Length::Fill),
                button(Text::new(t!("add_domain")))
                    .on_press(Message::ChangePage(Page::AddDomain))
                    .width(Length::Fill)
            ]
            .into()
        }));

    let actions = Row::new()
        .push(button("Add Domain").on_press(Message::ChangePage(Page::AddDomain)))
        .spacing(50)
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

pub fn help(_app: &App) -> Element<Message> {
    let title = text("Help")
        .width(Length::Shrink)
        .size(TITLE_SIZE)
        .align_x(Horizontal::Center);
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
struct DomainName {
    provider: String,
    name: String, // e.g. example.com ,
    dns_record: Vec<DnsRecord>,
}

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        DomainName {
            name: value,
            provider: String::from(""),
            dns_record: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    DomainPage,
    AddDomain,
    DnsRecord(DomainName),
    AddRecord,
    Help,
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::DomainPage => write!(f, "page_domain_manage"),
            Page::AddDomain => write!(f, "page_add_domain"),
            Page::DnsRecord(domain_name) => write!(f, "DnsRecord({})", domain_name.name),
            Page::AddRecord => write!(f, "{}", get_text("add_record")),
            Page::Help => write!(f, "Help"),
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

#[test]
fn it_counts_properly() {
    let mut app = App::default();

    app.update(Message::Increment);
    app.update(Message::Increment);
    app.update(Message::Decrement);

    assert_eq!(app.counter, 1);
}

fn get_text(name: &str) -> String {
    t!(name).into()
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
    }

    #[test]
    fn test_parse_json_config() {
        let config = Config::new_from_file("config.json");
        assert_eq!(config.name, "Domain Manager");
    }
}
