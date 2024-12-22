use crate::Page::AddDomain;
use iced::widget::{
    button, column, container, pick_list, row, stack, text, Button, Container, Text, TextInput,
};
use iced::window::Position;
use iced::{application, color, window, Color, Element, Length, Size, Theme};
use log::info;
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;

const TITLE_SIZE: u16 = 36;
const TITLE_PADDING: u16 = 20;
const CONTENT_SIZE: u16 = 20;

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

pub fn main() -> iced::Result {
    let app = App::default();
    app.start()
}

#[derive(Debug)]
struct App {
    theme: Theme,
    domain_names: Vec<DomainName>,
    counter: i32,
    current_page: Page,
    current_domain_name: Option<DomainName>,
    add_domain_field: AddDomainField,
}

impl App {
    pub fn start(&self) -> iced::Result {
        iced::application("Dns manager", Self::update, Self::view)
            .window(window::Settings {
                size: Size::new(720f32, 320f32), // start size
                position: Position::Default,
                min_size: None, // Some(ConfigWindow::MIN_SIZE.to_size()), // min size allowed
                max_size: None,
                visible: true,
                resizable: false,
                decorations: true,
                transparent: true,
                #[cfg(target_os = "linux")]
                platform_specific: PlatformSpecific {
                    application_id: String::from(SNIFFNET_LOWERCASE),
                    ..PlatformSpecific::default()
                },
                exit_on_close_request: true,
                ..Default::default()
            })
            .run()
    }
}

impl Default for App {
    /// 这里可以初始化一些数据，如域名列表
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
        }
    }
}

// 定义主题

impl App {
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
    let fruits = [
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
                let style = text::Style::default();
                style
            })
            .width(Length::Fill),
        TextInput::new("domain name", &app.add_domain_field.domain_name)
            .on_input(Message::AddDomainFormChanged),
        pick_list(fruits, state.provider, Message::DnsProviderSelected,)
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

/// 域名管理界面
fn domain_page(app: &App) -> Element<'static, Message> {
    container(
        // 返回到解析界面
        column(app.domain_names.iter().map(|domain_name| {
            // 这里是一行数据
            let counter = app.counter;
            row![
                text!("Counter{counter}").width(Length::Fill),
                text!("{}", domain_name.name).width(Length::Fill),
                text!("{}", domain_name.provider).width(Length::Fill),
                button("Add Dns")
                    .on_press(Message::DomainDeleted(domain_name.clone()))
                    .width(Length::Fill),
                button("Increment")
                    .on_press(Message::Increment)
                    .width(Length::Fill),
                button("Decrement")
                    .on_press(Message::Decrement)
                    .width(Length::Fill),
                button("dns list")
                    .on_press(Message::ChangePage(Page::DnsRecord(domain_name.clone())))
                    .width(Length::Fill),
                button("add domain")
                    .on_press(Message::ChangePage(Page::AddDomain))
                    .width(Length::Fill)
            ]
            .into()
        })),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
struct DnsRecord {
    domain_name: String,
    dns_name: String,
    dns_type: String, // A, AAAA, CNAME, MX, TXT, etc.
    dns_value: String,
}

#[test]
fn it_counts_properly() {
    let mut app = App::default();

    app.update(Message::Increment);
    app.update(Message::Increment);
    app.update(Message::Decrement);

    assert_eq!(app.counter, 1);
}
