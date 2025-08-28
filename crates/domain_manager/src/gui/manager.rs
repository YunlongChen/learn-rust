use crate::api::ali_api::{
    add_aliyun_dns_record, delete_aliyun_dns, query_aliyun_dns_list,
    query_aliyun_dns_operation_list,
};
use crate::api::dns_client::{DnsClient, DnsClientTrait};
use crate::api::model::dns_operate::RecordLog;
use crate::api::provider::aliyun::AliyunDnsClient;
use crate::configs::gui_config::{BackgroundConfig, LICENCE, WindowState};
use crate::gui::components::background::Background;
use crate::gui::components::console::{console_view, ConsoleState};
use crate::gui::components::footer::footer;
use crate::gui::components::header::header;
use crate::gui::model::domain::{DnsProvider, DnsRecord, Domain, DomainStatus};
use crate::gui::model::form::{AddDnsField, AddDomainField};
use crate::gui::model::gui::ReloadModel;
use crate::gui::pages::domain::{
    add_domain_page, add_domain_provider_page, AddDomainProviderForm, DomainProvider,
};
use crate::gui::pages::domain_dns_record::{add_dns_record, dns_record};
use crate::gui::pages::help::help;
use crate::gui::pages::names::Page;
use crate::gui::pages::types::settings::SettingsPage;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::gui::styles::ButtonType;
use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
use crate::gui::types::message::Message::ReloadComplete;
use crate::gui::types::message::{Message, SyncResult};
use crate::model::dns_record_response::Record;
use crate::models::account::{Account, NewAccount};
use crate::models::domain::NewDomain;
use crate::storage::records::get_records_by_domain;
use crate::storage::{
    add_domain_many, count_all_domains, create_account, delete_domain, delete_domain_by_account,
    get_account_domains, list_accounts, list_domains,
};
use crate::translations::types::language::Language;
use crate::translations::types::locale::Locale;
use crate::utils::types::icon::Icon;
use crate::utils::types::web_page::WebPage;
use crate::{get_text, Config, StyleType};
use iced::keyboard::Key;
use iced::widget::{
    button, center, container, horizontal_rule, horizontal_space, mouse_area, scrollable, text,
    Button, Column, Container, MouseArea, Row, Stack, Text, Tooltip,
};
use iced::Event::Window;
use iced::{
    keyboard, window, Alignment, Element, Font, Length, Point, Size, Subscription, Task, Theme,
};
use mockall::Any;
use sea_orm::DatabaseConnection;
use std::error::Error;
use std::sync::Mutex;
use std::{env, process};
use tokio::join;
use tracing::{debug, error, info, warn};

pub struct DomainManager {
    /// 应用程序的配置：设置、窗口属性、应用程序名称
    pub config: Config,
    /// 当前主题
    pub theme: Theme,
    pub domain_list: Vec<Domain>,
    /// 当前页面
    pub current_page: Page,
    pub current_domain_name: Option<Domain>,
    pub add_domain_field: AddDomainField,
    pub add_domain_provider_form: AddDomainProviderForm,
    pub domain_providers: Vec<DomainProvider>,
    pub last_page: Option<Page>,
    /// 查询进行中
    pub in_query: bool,
    /// dns列表
    pub dns_list: Vec<Record>, // 当前域名对应的DNS记录
    pub dns_log_list: Vec<RecordLog>, // 当前域名对应的DNS记录
    pub add_dns_form: AddDnsField,
    pub locale: Locale,
    /// 缩略图模式当前是否处于活动状态
    pub thumb_nail: bool,
    /// 未读通知数
    pub unread_notifications: usize,
    /// dns客户端
    pub dns_client: DnsClient,
    pub connection: Option<DatabaseConnection>,
    /// 客户端状态
    filter: Filter,
    pub search_query: String,
    dns_records: Vec<DnsRecord>,
    stats: DomainStats,
    is_syncing: bool,
    pub message: String,
    /// Toast通知相关字段
    pub toast_message: Option<String>,
    pub toast_visible: bool,
    /// 控制台状态
    pub console_state: ConsoleState,
    /// 悬浮窗状态
    pub floating_window_enabled: bool,
}

#[derive(Debug, Clone)]
struct DomainStats {
    total: u64,
    expiring: usize,
    providers: usize,
}

#[derive(Debug, Clone)]
struct Filter {
    pub selected_provider: Option<DomainProvider>,
    pub selected_domain: Option<Domain>,
}

impl Filter {
    pub fn reset(&mut self) {
        self.selected_provider = None;
        self.selected_domain = None
    }
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            selected_provider: None,
            selected_domain: None,
        }
    }
}

impl Default for DomainStats {
    fn default() -> Self {
        Self {
            total: 0,
            expiring: 0,
            providers: 0,
        }
    }
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
            ali_access_key_id: None,
            ali_access_key_secret: None,
            window_state: WindowState::default(),
            background_config: BackgroundConfig::default(),
        };

        // 初始化数据
        Self {
            current_page: Page::DomainPage,
            theme: Theme::Dark,
            domain_list: vec![],
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            add_dns_form: AddDnsField::default(),
            last_page: None,
            in_query: true,
            dns_list: vec![],
            dns_log_list: vec![],
            locale: Locale::Chinese,
            config,
            thumb_nail: false,
            unread_notifications: 0,
            dns_client: DnsClient::default(),
            connection: None,
            filter: Filter::default(),
            search_query: "".to_string(),
            dns_records: vec![],
            stats: DomainStats {
                total: 10,
                ..Default::default()
            },
            is_syncing: false,
            add_domain_provider_form: Default::default(),
            domain_providers: vec![],
            message: "加载中。。。".into(),
            toast_message: None,
            toast_visible: false,
            console_state: ConsoleState::default(),
            floating_window_enabled: false,
        }
    }
}

// 定义主题
impl DomainManager {
    fn locale(locale: Locale) {
        match locale {
            Locale::Chinese => rust_i18n::set_locale("zh_CN"),
            Locale::English => rust_i18n::set_locale("en"),
        }
    }

    pub fn new(config: Config, connection: DatabaseConnection) -> Self {
        // 初始化数据
        let domain_names = config.domain_names.clone();
        let locale: Locale = config.locale.clone().into();

        let dns_client: DnsClient = init_dns_client(&config).expect("Cannot init dns client.");
        info!("初始化dns_client 成功");
        let manager = Self {
            current_page: Page::DomainPage,
            theme: Theme::Light,
            domain_list: domain_names,
            current_domain_name: None,
            add_domain_field: AddDomainField::default(),
            last_page: None,
            in_query: false,
            config,
            thumb_nail: false,
            dns_list: vec![],
            dns_log_list: vec![],
            add_dns_form: AddDnsField::default(),
            locale,
            dns_client,
            connection: Some(connection),
            toast_message: None,
            toast_visible: false,
            ..DomainManager::default()
        };
        info!("初始化完成");
        manager
    }

    pub fn view(&self) -> Element<Message, StyleType> {
        let font = self.config.style_type.get_extension().font;
        // 整体布局：三列
        let header = header(self);

        // 保持锁的有效性
        let config = &self.config;
        let body: Element<Message, StyleType> = match self.current_page {
            Page::DomainPage => {
                Container::new(
                    Row::new()
                        .spacing(8) // 添加列间距
                        // 左侧托管商导航 - 固定宽度250px
                        .push(Self::provider_sidebar(self).width(Length::Fixed(250.0)))
                        // 中间域名列表 - 占据更多空间
                        .push(self.domain_list().width(Length::FillPortion(6)))
                        // 右侧详情面板 - 适中宽度
                        .push_maybe(match &self.filter.selected_domain {
                            Some(domain) => {
                                Some(self.domain_detail(domain).width(Length::FillPortion(4)))
                            }
                            None => {
                                Some(
                                    Container::new(
                                        center(
                                            Column::new()
                                                .spacing(10)
                                                .push(text("🔍").size(48))
                                                .push(text("选择域名查看详情").size(16))
                                                .push(text("点击左侧域名列表中的任意域名").size(12))
                                                .align_x(Alignment::Center)
                                        )
                                    )
                                    .width(Length::FillPortion(4))
                                    .height(Length::Fill)
                                    .class(ContainerType::Bordered)
                                )
                            }
                        })
                        .height(Length::Fill)
                        .width(Length::Fill),
                )
                .padding(8) // 添加整体内边距
                .class(ContainerType::Standard) // 改为透明容器以显示背景
                .into()
            }
            Page::AddDomain => add_domain_page(self).into(),
            Page::DnsRecord => dns_record(self).into(),
            Page::AddRecord => add_dns_record(self).into(),
            Page::Help => help(self).into(),
            Page::AddProvider => add_domain_provider_page(self).into(),
            Page::Settings(settings_page) => crate::gui::pages::settings::settings_page(self, settings_page).into(),
            Page::Console => console_view(&self.console_state, font).into(),
            _ => help(self).into(),
        };

        // 底部
        let footer = footer(
            false,
            config.language,
            config.color_gradient,
            config.style_type.get_extension().font,
            config.style_type.get_extension().font_headers,
            &Mutex::new(Some(true)),
        );

        // 主要内容 - 使用透明容器以显示背景
        let main_content = Column::new()
            .push(header)
            .push(
                Container::new(body)
                    .height(Length::Fill)
                    .class(ContainerType::Standard) // 使用透明容器
            )
            .push(footer)
            .into();

        // 如果有背景，则创建带背景的容器
        let content_with_background = if self.config.background_config.background_type != crate::configs::gui_config::BackgroundType::None {
            // 使用Stack来叠加背景和内容
            iced::widget::Stack::new()
                .push(Background::new(
                    self.config.background_config.background_type.clone(),
                    self.config.background_config.opacity,
                ).view())
                .push(
                    Container::new(main_content)
                        .class(ContainerType::Standard) // 确保主容器也是透明的
                )
                .into()
        } else {
            main_content
        };

        // 添加toast通知
        crate::gui::components::toast::with_toast(
            content_with_background,
            self.toast_message.as_deref().unwrap_or(""),
            self.toast_visible,
        )
    }

    // 左侧托管商导航
    fn provider_sidebar(app: &DomainManager) -> Container<Message, StyleType> {
        let provider_list = Column::new().spacing(8).width(Length::Fill);
        debug!("托管商数量：「{}」", app.domain_providers.len());

        let provider_list = app
            .domain_providers
            .iter()
            .fold(provider_list, |col, provider| {
                let is_selected = app.filter.selected_provider.as_ref() == Some(provider);
                col.push(provider_item(provider, is_selected))
            });

        // 添加"全部"选项
        let all_providers_item = Container::new(
            button(
                Row::new()
                    .spacing(8)
                    .push(text("📁").size(14))
                    .push(text("全部托管商").size(14))
                    .align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .on_press(Message::ProviderSelected(None))
            .class(if app.filter.selected_provider.is_none() {
                ButtonType::Primary
            } else {
                ButtonType::Standard
            })
        )
        .width(Length::Fill)
        .padding([4, 0]);

        let sidebar = Column::new()
            .spacing(12)
            .push(
                Row::new()
                    .spacing(8)
                    .push(text("🏢").size(16))
                    .push(Text::new("托管商").size(16).width(Length::Fill))
                    .align_y(Alignment::Center)
            )
            .push(horizontal_rule(1))
            .push(all_providers_item)
            .push(scrollable(provider_list).height(Length::Fill));

        container(sidebar)
            .height(Length::Fill)
            .padding(12)
            .class(ContainerType::Bordered)
            .into()
    }

    // 中间域名列表
    fn domain_list(&self) -> Container<Message, StyleType> {
        let font: Font = self.config.style_type.get_extension().font;

        let title = match &self.filter.selected_provider {
            None => "域名管理".to_string(),
            Some(provider) => {
                format!(
                    "{} 域名 [{}]",
                    provider.provider_name,
                    provider.provider.name()
                )
            }
        };
        //https://jsd.nn.ci/gh/YunlongChen/yunlongchen@main/out/github-snake-dark.svg

        let header: Row<Message, StyleType> = Row::new()
            .spacing(12)
            .align_y(Alignment::Center)
            .push(
                Row::new()
                    .spacing(8)
                    .push(text("📋").size(18))
                    .push(Text::new(title).size(18))
                    .align_y(Alignment::Center)
            )
            .push(horizontal_space())
            .push(
                Row::new()
                    .spacing(8)
                    .push(
                        button(
                            Row::new()
                                .spacing(6)
                                .push(text("🔄").size(12))
                                .push(text("刷新").size(12))
                                .align_y(Alignment::Center)
                        )
                        .on_press(Message::Reset)
                        .class(ButtonType::Standard)
                        .padding([6, 12])
                    )
                    .push(
                        button(
                            Row::new()
                                .spacing(6)
                                .push(text("🎭").size(12))
                                .push(text("模拟").size(12))
                                .align_y(Alignment::Center)
                        )
                        .on_press(Message::Mock)
                        .class(ButtonType::Standard)
                        .padding([6, 12])
                    )
                    .push(
                        button(
                            Row::new()
                                .spacing(6)
                                .push(text("☁️").size(12))
                                .push(text(if self.is_syncing { "同步中..." } else { "同步" }).size(12))
                                .align_y(Alignment::Center)
                        )
                        .on_press(Message::Sync)
                        .class(if self.is_syncing { ButtonType::Standard } else { ButtonType::Primary })
                        .padding([6, 12])
                    )
                    .align_y(Alignment::Center)
            )
            .padding(12);

        // 统计卡片 - 更紧凑的设计
        let stats = Row::new()
            .spacing(8)
            .push(stat_card(
                "总计".to_string(),
                self.stats.total.to_string(),
                "域名总数",
            ))
            .push(stat_card(
                "即将到期".to_string(),
                self.stats.expiring.to_string(),
                "30天内",
            ))
            .push(stat_card(
                "托管商".to_string(),
                self.stats.providers.to_string(),
                "已配置",
            ))
            .width(Length::Fill);

        // 过滤域名列表
        let filtered_domains: Vec<&Domain> = self
            .domain_list
            .iter()
            .filter(|domain| match &self.filter.selected_provider {
                Some(provider) => domain.provider == provider.provider,
                None => true,
            })
            .collect();

        debug!("域名数量：「{}」，过滤后：「{}」", self.domain_list.len(), filtered_domains.len());

        // 域名列表内容
        let domain_list_content = if filtered_domains.is_empty() {
            Container::new(
                center(
                    Column::new()
                        .spacing(12)
                        .push(text("📭").size(48))
                        .push(text("暂无域名").size(16))
                        .push(text("点击同步按钮从云端获取域名数据").size(12))
                        .align_x(Alignment::Center)
                )
            )
            .height(Length::Fill)
            .width(Length::Fill)
        } else {
            let domain_list = filtered_domains
                .iter()
                .enumerate()
                .fold(Column::new().spacing(4), |column, (_index, domain)| {
                    let is_selected = self.filter.selected_domain == Some((*domain).clone());
                    column.push(domain_row(domain, is_selected, font))
                });

            Container::new(scrollable(domain_list).height(Length::Fill))
                .height(Length::Fill)
                .width(Length::Fill)
        };

        let content = Column::new()
            .spacing(12)
            .push(header)
            .push(horizontal_rule(1))
            .push(stats)
            .push(horizontal_rule(1))
            .push(domain_list_content);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .class(ContainerType::Bordered)
    }

    /// 创建域名详情面板
    ///
    /// # 参数
    /// * `domain` - 域名信息
    fn domain_detail<'a>(&'a self, domain: &'a Domain) -> Container<'a, Message, StyleType> {
        // 域名标题和状态
        let status_icon = match domain.status {
            DomainStatus::Active => "🟢",
            DomainStatus::Suspended => "🔴",
            DomainStatus::Warning => "🟡",
        };

        let provider_icon = match domain.provider.name() {
            "阿里云" => "☁️",
            "腾讯云" => "🌐",
            "华为云" => "🔧",
            _ => "🏢",
        };

        let domain_title = Container::new(
            Column::new()
                .spacing(8)
                .push(
                    Row::new()
                        .spacing(8)
                        .push(text("🌐").size(20))
                        .push(Text::new(&domain.name).size(18))
                        .align_y(Alignment::Center)
                )
                .push(
                    Row::new()
                        .spacing(6)
                        .push(text(provider_icon).size(14))
                        .push(Text::new(domain.provider.name()).size(12))
                        .push(text(status_icon).size(14))
                        .push(Text::new(domain.status.text()).size(12))
                        .align_y(Alignment::Center)
                )
        )
        .padding(16)
        .class(ContainerType::Bordered);

        // 域名基本信息
        let domain_info = Container::new(
            Column::new()
                .spacing(12)
                .push(
                    Row::new()
                        .spacing(8)
                        .push(text("📋").size(14))
                        .push(Text::new("基本信息").size(14))
                        .align_y(Alignment::Center)
                )
                .push(info_row("📅 注册日期", "2020-08-15"))
                .push(info_row("⏰ 到期日期", &domain.expiry))
                .push(info_row("🌍 域服务器", &domain.name))
                .push(info_row("📊 域名状态", domain.status.text()))
        )
        .padding(16)
        .class(ContainerType::Bordered);

        // 托管商特色功能
        let mut features = Row::new().spacing(8);
        for feature in domain.provider.features() {
            features = features.push(
                button(
                    Row::new()
                        .spacing(4)
                        .push(text("⚡").size(10))
                        .push(text(feature).size(10))
                        .align_y(Alignment::Center)
                )
                .class(ButtonType::Standard)
                .padding([6, 12])
                .on_press(Message::FeatureClicked(feature.to_string()))
            );
        }

        let features_section = Container::new(
            Column::new()
                .spacing(8)
                .push(
                    Row::new()
                        .spacing(8)
                        .push(text("🚀").size(14))
                        .push(Text::new("特色功能").size(14))
                        .align_y(Alignment::Center)
                )
                .push(features)
        )
        .padding(16)
        .class(ContainerType::Bordered);

        // DNS记录管理
        let dns_header = Row::new()
            .spacing(10)
            .push(
                Row::new()
                    .spacing(8)
                    .push(text("🔧").size(14))
                    .push(Text::new("DNS记录管理").size(14))
                    .align_y(Alignment::Center)
            )
            .push(horizontal_space().width(Length::Fill))
            .push(
                button(
                    Row::new()
                        .spacing(4)
                        .push(text("🔍").size(10))
                        .push(text("查询").size(10))
                        .align_y(Alignment::Center)
                )
                .class(ButtonType::Standard)
                .padding([4, 8])
                .on_press(Message::AddDnsRecord)
            )
            .push(
                button(
                    Row::new()
                        .spacing(4)
                        .push(text("➕").size(10))
                        .push(text("添加").size(10))
                        .align_y(Alignment::Center)
                )
                .class(ButtonType::Primary)
                .padding([4, 8])
                .on_press(Message::AddDnsRecord)
            )
            .align_y(Alignment::Center);

        let dns_table = Column::new().spacing(5);

        let dns_table = self
            .dns_records
            .iter()
            .enumerate()
            .fold(dns_table, |col, (index, record)| {
                col.push(dns_row(record, index))
            });

        let dns_section = Container::new(
            Column::new()
                .spacing(12)
                .push(dns_header)
                .push(horizontal_rule(1))
                .push(scrollable(dns_table))
        )
        .padding(16)
        .class(ContainerType::Bordered);

        let content = Column::new()
            .spacing(16)
            .push(domain_title)
            .push(domain_info)
            .push(features_section)
            .push(dns_section);

        container(scrollable(content))
            .width(Length::Fixed(400.0))
            .height(Length::Fill)
            .padding(8)
            .class(ContainerType::Background)
            .padding(10)
            .class(ContainerType::BorderedRound)
            .into()
    }

    pub fn get_custom_button<'a>(
        font: Font,
        language: Language,
        open_overlay: SettingsPage,
        message: Message,
        icon: Icon,
        title: String,
    ) -> Tooltip<'a, Message, StyleType> {
        let content = button(
            icon.to_text()
                .size(20)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .padding(0)
        .height(40)
        .width(60)
        .on_press(message);

        Tooltip::new(
            content,
            Text::new(title.clone()).font(font),
            iced::widget::tooltip::Position::Left,
        )
        .gap(5)
        .class(ContainerType::Tooltip)
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        debug!(
            "是否最小化:{:?},未读通知：{:?}",
            self.thumb_nail, self.unread_notifications
        );
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
            Message::Mock => {
                self.handle_reset();
                let mock_data = self.mock_data();
                let (providers, domains, records) = mock_data;

                let total_size = *(&domains.len()) as u64;

                self.update(ReloadComplete(ReloadModel::new_from(
                    providers, domains, records, total_size,
                )))
            }

            Message::Reset => {
                self.handle_reset();
                self.update(Message::Reload)
            }
            Message::Reload => {
                info!("收到界面刷新消息");
                debug!("当前选择的域名托管商: {:?}", self.filter.selected_provider);
                // 更新数据
                // TODO 这里可能会影响界面刷新，需要在异步线程里面完成
                match &self.connection {
                    None => {
                        error!("当前没有数据库连接，无法刷新界面数据");
                        self.message = "数据库连接失败，无法加载数据".to_string();
                        self.update(ReloadComplete(ReloadModel::default()))
                    }
                    Some(connection) => {
                        // 克隆连接，因为我们需要将它移动到异步任务中
                        let conn = connection.clone();
                        let clone_filter = self.filter.clone();

                        Task::perform(Self::handle_reload(conn, clone_filter), |result| {
                            match result {
                                Ok(result) => {
                                    let (accounts, domains, total_count, message) = result;
                                    info!(
                                        "数据加载成功，账户: {}, 域名: {}, 总数: {}",
                                        accounts.len(),
                                        domains.len(),
                                        total_count
                                    );
                                    let providers = accounts
                                        .into_iter()
                                        .map(|account| account.into())
                                        .collect();

                                    return ReloadComplete(ReloadModel {
                                        reload_types: vec![],
                                        providers,
                                        domains,
                                        records: vec![],
                                        message: "".to_string(),
                                        total_count,
                                    });
                                }
                                Err(err) => {
                                    error!("数据加载失败: {:?}", err);
                                }
                            }
                            ReloadComplete(ReloadModel::default())
                        })
                    }
                }
            }
            ReloadComplete(result) => {
                info!("数据重新加载完成！，当前加载数据类型：「所有」");
                // 创建上面的
                self.domain_providers = result.providers;
                self.domain_list = result.domains;
                self.dns_records = result.records;
                self.stats.total = result.total_count;
                // 清除加载消息
                self.message = result.message;
                Task::none()
            }
            Message::Started => {
                info!("Application Started!");
                let _ = self.update(Message::ChangeLocale(Locale::Chinese));
                self.update(Message::Reload)
            }
            Message::ChangeLocale(locale) => {
                Self::locale(locale);
                self.update(Message::LocaleChanged(locale))
            }
            Message::LocaleChanged(locale) => {
                self.locale = locale;
                // 更新配置中的语言设置
                self.config.locale = match locale {
                    Locale::Chinese => "zh_CN".to_string(),
                    Locale::English => "en".to_string(),
                };
                self.config.language = match locale {
                    Locale::Chinese => Language::ZH,
                    Locale::English => Language::EN,
                };
                // 保存配置到文件
                if let Err(e) = self.config.save_to_file("config.json") {
                    error!("保存语言配置失败: {}", e);
                }
                info!("语言切换为: {:?}", locale);
                Task::none()
            }
            Message::ToggleTheme => {
                if self.theme == Theme::TokyoNightLight {
                    self.theme = Theme::SolarizedDark
                } else {
                    self.theme = Theme::TokyoNightLight
                }
                info!("修改主题为{}", &self.theme);
                Task::none()
            }
            Message::DomainSelected(domain) => {
                self.filter.selected_domain = Some(domain);
                Task::none()
            }
            Message::SearchChanged(search_content) => {
                info!("搜索内容:{}", &search_content);
                self.search_query = search_content;
                Task::none()
            }
            Message::AddProviderFormProviderChanged(dns_provider) => {
                self.add_domain_provider_form.provider = Some(dns_provider.clone());
                self.add_domain_provider_form.credential = Some(dns_provider.credential());
                Task::none()
            }
            Message::AddProviderFormNameChanged(name) => {
                debug!("域名托管商的名称发生了变化：「{}」", &name);
                self.add_domain_provider_form.provider_name = name;
                Task::none()
            }
            Message::AddProviderFormCredentialChanged(credential) => {
                self.handle_add_provider_form_credential_changed(credential)
            }
            Message::ProviderSelected(dns_provider) => self.handle_provider_selected(dns_provider),
            Message::Sync => self.handle_sync(),
            Message::SyncAllDomains => self.handle_sync_domain(),
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
                Task::none()
            }
            Message::AddDnsProvider => {
                self.add_domain_provider_form.clear();
                self.update(Message::ChangePage(Page::AddProvider))
            }
            Message::ValidateCredential => {
                info!("对凭证进行校验");
                Task::none()
            }
            Message::AddCredential => self.add_credential(),
            Message::DnsProviderChange => self.update(Message::Reload),
            Message::QueryDnsResult(dns_list) => {
                // 将查询到的Record转换为DnsRecord并更新UI显示字段
                self.dns_records = dns_list.into_iter().map(|record| {
                    DnsRecord {
                        name: record.rr,
                        record_type: record.record_type.get_value().to_string(),
                        value: record.value,
                        ttl: record.ttl.to_string(),
                    }
                }).collect();
                info!("DNS记录更新完成，共 {} 条记录", self.dns_records.len());
                Task::none()
            }
            Message::QueryDnsLogResult(logs) => {
                info!("dns操作日志查询成功");
                self.dns_log_list = logs;
                Task::none()
            }
            Message::DomainDeleted(domain_name) => {
                info!("删除域名：domain_name:{}", domain_name.name);
                Task::none()
            }
            Message::AddDomainFormChanged(domain_name) => {
                self.add_domain_field.domain_name = domain_name;
                Task::none()
            }
            Message::SubmitDomainForm => {
                info!(
                    "提交域名表单：添加完毕，域名名称：「{}」,托管商类型：「{}」",
                    &self.add_domain_field.domain_name,
                    &match self.add_domain_field.provider {
                        Some(x) => x,
                        None => todo!(),
                    }
                    .name()
                );
                self.update(Message::ChangePage(Page::DomainPage))
            }
            Message::QueryDomainDnsRecord(domain_name) => {
                self.current_domain_name = Some(domain_name.clone());
                // 异步加载选定域名的DNS记录
                let domain_name_for_query = domain_name.name.clone();
                Task::batch([
                    self.update(Message::ChangePage(Page::DnsRecord)),
                    Task::perform(
                        Self::handle_dns_reload(domain_name_for_query),
                        Message::QueryDnsResult,
                    ),
                ])
            }
            Message::DnsProviderSelected(provider) => {
                self.add_domain_field.provider = Some(provider);
                self.update(Message::Reload)
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
                self.last_page = last_page;
                self.update(Message::ChangePage(Page::Help))
            }
            Message::CloseHelp => match &self.last_page {
                Some(page) => self.update(Message::ChangePage(page.clone())),
                _ => Task::none(),
            },
            Message::QueryDomainResult(domain_names, provider) => {
                self.in_query = false;

                match &self.connection {
                    None => Task::none(),
                    Some(conn) => {
                        let clone_connection = conn.clone();
                        let clone_provider = provider.clone();

                        let add_domain_list: Vec<NewDomain> = domain_names
                            .into_iter()
                            .map(|domain_name| NewDomain {
                                domain_name: domain_name.name,
                                registration_date: None,
                                expiration_date: None,
                                registrar: None,
                                status: crate::models::domain::DomainStatus::Active,
                                account_id: clone_provider.account_id,
                            })
                            .collect();

                        Task::perform(
                            async move {
                                let _ = delete_domain_by_account(
                                    &clone_connection.clone(),
                                    clone_provider.account_id,
                                )
                                .await
                                .expect("执行异常");

                                let _ = add_domain_many(&clone_connection, add_domain_list)
                                    .await
                                    .expect("执行异常");

                                Ok(())
                            },
                            |_result: Result<(), Box<dyn Error + Send>>| Message::Reload,
                        )
                    }
                }
            }
            Message::DnsDelete(record_id) => {
                info!("删除dns记录:{}", &record_id);
                Task::perform(Self::handle_dns_record_delete(record_id), |response| {
                    info!("请求接口信息:{:?}", response);
                    match response {
                        None => Message::ChangePage(Page::DnsRecord),
                        Some(record_id) => Message::DnsRecordDeleted(record_id.clone()),
                    }
                })
            }
            Message::AddDnsRecord => match &self.current_domain_name {
                Some(domain_name) => {
                    let name = &domain_name.name;
                    self.add_dns_form = AddDnsField {
                        domain_name: name.to_string(),
                        ..AddDnsField::default()
                    };
                    self.update(Message::ChangePage(Page::AddRecord))
                }
                None => Task::none(),
            },
            Message::DnsFormNameChanged(record_name) => {
                info!("添加dns记录表单变化：:{}", &record_name);
                self.add_dns_form = AddDnsField {
                    record_name,
                    ..self.add_dns_form.clone()
                };
                Task::none()
            }
            Message::AddDnsFormSubmit => match self.add_dns_form.validate() {
                true => Task::perform(
                    Self::handle_dns_record_add(AddDnsField {
                        ..self.add_dns_form.clone()
                    }),
                    |domain_names| {
                        info!("请求接口信息:{:?}", domain_names);
                        Message::ChangePage(Page::AddRecord)
                    },
                ),
                false => Task::none(),
            },
            Message::DnsFormRecordTypeChanged(record_type) => {
                // info!("添加dns记录表单变化：", &record_type);
                self.handle_dns_add(AddDnsField {
                    record_type: Some(record_type),
                    ..self.add_dns_form.clone()
                });
                Task::none()
            }
            Message::DnsFormValueChanged(value) => {
                // info!("添加dns记录表单变化：", &value);
                self.handle_dns_add(AddDnsField {
                    value,
                    ..self.add_dns_form.clone()
                });
                Task::none()
            }
            Message::DnsFormTtlChanged(ttl) => {
                // info!("添加dns记录表单变化：", ttl);
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
            Message::OpenWebPage(web_page) => {
                Self::open_web(&web_page);
                Task::none()
            }
            Message::SyncAllDomainsComplete(result) => {
                info!("收到同步域名完成消息，结果: {:?}", result);
                self.is_syncing = false;
                match result {
                    SyncResult::Success => {
                        info!("域名同步成功，准备刷新界面");
                        self.message = "".to_string(); // 清除错误消息
                        self.update(Message::Reload) // 触发界面刷新
                    }
                    SyncResult::Failed(err) => {
                        error!("域名同步失败，错误信息: {}", err);
                        self.message = format!("同步失败: {}", err);
                        Task::none()
                    }
                    SyncResult::Cancelled => {
                        info!("域名同步被取消");
                        Task::none()
                    }
                }
            }
            Message::DragWindow => {
                // 获取最旧的窗口并拖动
                window::get_oldest().then(|id_option| {
                    if let Some(id) = id_option {
                        Task::done(Message::StartDragWindow(id))
                    } else {
                        Task::none()
                    }
                })
            }
            Message::StartDragWindow(id) => {
                // 开始拖动指定窗口
                window::drag(id)
            }
            Message::WindowMoved(x, y) => {
                // 处理窗口移动事件，更新配置中的窗口位置
                info!("窗口移动到位置: ({}, {})", x, y);
                self.config.update_window_state(x, y, self.config.window_state.width, self.config.window_state.height);
                // 保存配置到文件
                if let Err(e) = self.config.save_to_file("config.json") {
                    error!("保存窗口位置配置失败: {}", e);
                }
                Task::none()
            }
            Message::WindowResized(width, height) => {
                // 处理窗口大小调整事件，更新配置中的窗口大小
                info!("窗口大小调整为: {}x{}", width, height);
                self.config.update_window_state(self.config.window_state.x, self.config.window_state.y, width, height);
                // 保存配置到文件
                if let Err(e) = self.config.save_to_file("config.json") {
                    error!("保存窗口大小配置失败: {}", e);
                }
                Task::none()
            }
            Message::WindowMinimize => {
                // 处理窗口最小化事件
                info!("窗口最小化");
                window::get_oldest().then(|id_option| {
                    if let Some(id) = id_option {
                        window::minimize(id, true)
                    } else {
                        Task::none()
                    }
                })
            }
            Message::WindowMaximize => {
                // 处理窗口最大化/还原事件
                info!("窗口最大化/还原");
                window::get_oldest().then(|id_option| {
                    if let Some(id) = id_option {
                        window::toggle_maximize(id)
                    } else {
                        Task::none()
                    }
                })
             }
             Message::ChangeBackground(background_type) => {
                // 处理背景切换事件
                info!("切换背景类型: {:?}", background_type);
                self.config.background_config.background_type = background_type;
                // 保存配置到文件
                if let Err(e) = self.config.save_to_file("config.json") {
                    error!("保存背景配置失败: {}", e);
                }
                Task::none()
             }
             Message::OpenSettings(settings_page) => {
                // 处理打开设置页面事件
                info!("打开设置页面: {:?}", settings_page);
                self.last_page = Some(self.current_page.clone());
                self.current_page = Page::Settings(settings_page);
                Task::none()
             }
             Message::BackgroundOpacityChanged(opacity) => {
                // 处理背景透明度改变事件
                info!("背景透明度改变: {}", opacity);
                self.config.background_config.opacity = opacity.clamp(0.0, 1.0);
                // 保存配置到文件
                if let Err(e) = self.config.save_to_file("config.json") {
                    error!("保存背景透明度配置失败: {}", e);
                }
                Task::none()
             }
             Message::ShowToast(message) => {
                self.toast_message = Some(message.clone());
                self.toast_visible = true;
                // 3秒后自动隐藏toast
                Task::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::HideToast,
                )
             }
             Message::HideToast => {
                self.toast_visible = false;
                self.toast_message = None;
                Task::none()
             }
             Message::ChangeConsoleTab(tab) => {
                self.console_state.current_tab = tab;
                Task::none()
             }
             Message::ClearConsoleLogs => {
                self.console_state.clear_logs();
                self.toast_message = Some("控制台日志已清空".to_string());
                self.toast_visible = true;
                // 3秒后自动隐藏toast
                Task::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::HideToast,
                )
             }
             /// 切换悬浮窗模式
             Message::ToggleFloatingWindow => {
                self.floating_window_enabled = !self.floating_window_enabled;
                let message = if self.floating_window_enabled {
                    get_text("floating_window_enabled")
                } else {
                    get_text("floating_window_disabled")
                };
                self.toast_message = Some(message);
                self.toast_visible = true;
                info!("悬浮窗模式切换为: {}", self.floating_window_enabled);
                // 3秒后自动隐藏toast
                Task::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::HideToast,
                )
             }
             /// 创建悬浮窗
             Message::CreateFloatingWindow => {
                self.toast_message = Some(get_text("floating_window_created"));
                self.toast_visible = true;
                info!("创建悬浮窗请求");
                // 3秒后自动隐藏toast
                Task::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::HideToast,
                )
             }
             /// 关闭悬浮窗
             Message::CloseFloatingWindow => {
                self.floating_window_enabled = false;
                self.toast_message = Some(get_text("floating_window_closed"));
                self.toast_visible = true;
                info!("关闭悬浮窗");
                // 3秒后自动隐藏toast
                Task::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::HideToast,
                )
             }
             _ => {
                debug!("未处理的消息：{:?}", message);
                Task::none()
            }
        }
    }

    async fn sync_domains(app: DnsClient) -> Vec<Domain> {
        info!("开始同步域名信息，使用DNS客户端: {:?}", app);
        let mut all_domains: Vec<Domain> = Vec::new();

        // 1. 获取所有域名信息
        info!("调用DNS客户端获取所有域名信息");
        let domain_name_response = app.get_all_domain_info().await;
        match domain_name_response {
            Ok(domain_names) => {
                info!(
                    "同步域名信息成功，总共同步了「{}」条域名记录",
                    domain_names.len()
                );
                for (i, domain) in domain_names.iter().enumerate() {
                    debug!("域名 {}/{}: {}", i + 1, domain_names.len(), domain.name);
                }
                // 将域名添加到结果列表
                all_domains.extend(domain_names);
            }
            Err(err) => {
                error!("获取域名异常: {}，详细错误: {:?}", err, err);
            }
        }
        info!("域名同步完成，返回 {} 个域名", all_domains.len());
        all_domains
    }

    async fn handle_domain_reload(provider: DomainProvider) -> (Vec<Domain>, DomainProvider) {
        let domains: Vec<Domain> = vec![];
        info!(
            "开始查询域名列表，提供商: {}, 类型: {}",
            provider.provider_name,
            provider.provider.name()
        );
        match provider.provider {
            DnsProvider::Aliyun => {
                let credential = &provider.credential;

                info!(
                    "正在查询托管商：「{}」的域名信息,托管商类型：「{}」",
                    &provider.provider_name,
                    &provider.provider.name()
                );

                match credential {
                    Credential::ApiKey(apikey_credential) => {
                        debug!("使用API密钥认证方式，密钥ID: {}", apikey_credential.api_key);
                        let aliyun_dns_client = AliyunDnsClient::new(
                            apikey_credential.api_key.clone(),
                            apikey_credential.api_secret.clone(),
                        );
                        info!("创建阿里云DNS客户端成功: {:?}", &aliyun_dns_client);

                        info!("开始查询阿里云域名列表");
                        let result = aliyun_dns_client.list_domains(0, 100).await;

                        let mut dns_records: Vec<DnsRecord> = vec![];

                        match result {
                            Ok(domain_names) => {
                                info!("成功获取阿里云域名列表，共 {} 个域名", domain_names.len());
                                for (i, domain_name) in domain_names.iter().enumerate() {
                                    info!(
                                        "处理域名 {}/{}: {}",
                                        i + 1,
                                        domain_names.len(),
                                        domain_name.name
                                    );
                                    let dns_record_response = aliyun_dns_client
                                        .list_dns_records(domain_name.name.clone())
                                        .await;

                                    match dns_record_response {
                                        Ok(records) => {
                                            info!(
                                                "查询域名:{}的解析列表成功：解析数量：「{}」",
                                                &domain_name.name,
                                                records.len()
                                            );

                                            for (j, record) in records.iter().enumerate() {
                                                debug!(
                                                    "处理DNS记录 {}/{}: 类型={}, 值={}",
                                                    j + 1,
                                                    records.len(),
                                                    record.record_type,
                                                    record.value
                                                );
                                                dns_records.push(DnsRecord {
                                                    name: record.value.clone(),
                                                    record_type: record.record_type.to_string(),
                                                    value: record.value.clone(),
                                                    ttl: record.ttl.to_string(),
                                                })
                                            }
                                        }
                                        Err(err) => {
                                            error!(
                                                "查询域名 {} 解析列表失败：「{:?}」",
                                                domain_name.name, err
                                            )
                                        }
                                    }
                                }

                                info!(
                                    "获取到了【{}】条域名记录,{}条域名解析记录！",
                                    domain_names.len(),
                                    dns_records.len()
                                );
                                (
                                    domain_names
                                        .into_iter()
                                        .map(|domain_name| {
                                            debug!("转换域名: {}", domain_name.name);
                                            Domain {
                                                name: domain_name.name,
                                                ..Default::default()
                                            }
                                        })
                                        .collect(),
                                    provider,
                                )
                            }
                            Err(err) => {
                                error!(
                                    "同步阿里云域名信息发生异常: {:?}，详细信息: {:?}",
                                    err, err
                                );
                                (vec![], provider)
                            }
                        }
                    }
                    _ => {
                        error!(
                            "认证方式错误: 阿里云的认证方式应该是apiKey，但收到了: {:?}",
                            credential
                        );
                        (vec![], provider)
                    }
                }
            }
            _ => {
                error!("当前认证方式未实现: {}，无法处理", provider.provider.name());
                (vec![], provider)
            }
        }
    }

    /// 异步加载指定域名的DNS记录
    ///
    /// # 参数
    /// * `domain_name` - 要查询DNS记录的域名
    ///
    /// # 返回值
    /// 返回DNS记录列表，如果查询失败则返回空列表
    async fn handle_dns_reload(domain_name: String) -> Vec<Record> {
        info!("开始查询域名DNS记录: {}", domain_name);

        // 从环境变量获取阿里云认证信息
        match (env::var("ALIBABA_CLOUD_ACCESS_KEY_ID"), env::var("ALIBABA_CLOUD_ACCESS_KEY_SECRET")) {
            (Ok(access_key_id), Ok(access_key_secret)) => {
                let aliyun_dns_client = AliyunDnsClient::new(access_key_id, access_key_secret);

                match aliyun_dns_client.list_dns_records(domain_name.clone()).await {
                    Ok(records) => {
                        info!("成功获取域名 {} 的DNS记录，共 {} 条", domain_name, records.len());
                        records
                    }
                    Err(err) => {
                        error!("查询域名 {} 的DNS记录失败: {:?}", domain_name, err);
                        vec![]
                    }
                }
            }
            _ => {
                error!("未找到阿里云认证信息，请设置环境变量 ALIBABA_CLOUD_ACCESS_KEY_ID 和 ALIBABA_CLOUD_ACCESS_KEY_SECRET");
                vec![]
            }
        }
    }

    async fn handle_dns_operate_log_query(domain_name: String) -> Vec<RecordLog> {
        info!("查询域名信息");
        let dns_operate_logs = query_aliyun_dns_operation_list(domain_name);
        dns_operate_logs
    }

    async fn handle_dns_record_add(domain_name: AddDnsField) -> bool {
        info!("添加域名解析记录");
        add_aliyun_dns_record(&domain_name)
    }

    async fn handle_dns_record_delete(record_id: String) -> Option<String> {
        info!("删除域名解析记录");
        delete_aliyun_dns(record_id)
    }

    /// 根据当前主题状态返回对应的StyleType
    pub(crate) fn theme(&self) -> StyleType {
        match self.theme {
            Theme::Light => StyleType::Day,
            Theme::Dark => StyleType::Night,
            Theme::TokyoNightLight => StyleType::MonAmour,
            Theme::SolarizedDark => StyleType::DeepSea,
            _ => StyleType::default(),
        }
    }

    fn open_web(web_page: &WebPage) {
        let url = web_page.get_url();

        #[cfg(target_os = "windows")]
        let cmd = "explorer";
        #[cfg(target_os = "macos")]
        let cmd = "open";
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        let cmd = "xdg-open";

        process::Command::new(cmd)
            .arg(url)
            .spawn()
            .unwrap()
            .wait()
            .unwrap_or_default();
    }

    fn handle_dns_add(&mut self, form: AddDnsField) {
        self.add_dns_form = form;
    }

    // 监听键盘
    pub(crate) fn keyboard_subscription(_: &DomainManager) -> Subscription<Message> {
        info!("创建键盘监听");
        let key = keyboard::on_key_press(|key, _| {
            info!("监听到键盘事件：{:?}", &key);
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

    fn mock_data(&mut self) -> (Vec<DomainProvider>, Vec<Domain>, Vec<DnsRecord>) {
        info!("添加Mock数据！");
        // 初始化提供程序列表
        let dns_provider_list = vec![DomainProvider {
            account_id: 1,
            provider_name: "".to_string(),
            provider: DnsProvider::Aliyun,
            credential: Credential::UsernamePassword(UsernamePasswordCredential {
                username: "测试账号".to_string(),
                password: "测试密码".to_string(),
            }),
        }];

        // 初始化DNS记录
        let domain_list = vec![
            Domain {
                id: None,
                name: "example.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
                records: vec![],
            },
            Domain {
                id: None,
                name: "example2.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
                records: vec![],
            },
            Domain {
                id: None,
                name: "example3.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
                records: vec![],
            },
            Domain {
                id: None,
                name: "example4.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
                records: vec![],
            },
        ];

        let dns_records = vec![
            DnsRecord {
                record_type: "A".to_string(),
                name: "@".to_string(),
                value: "192.0.2.1".to_string(),
                ttl: "自动".to_string(),
            },
            DnsRecord {
                record_type: "A".to_string(),
                name: "www".to_string(),
                value: "192.0.2.1".to_string(),
                ttl: "自动".to_string(),
            },
            DnsRecord {
                record_type: "CNAME".to_string(),
                name: "mail".to_string(),
                value: "mailprovider.com".to_string(),
                ttl: "1小时".to_string(),
            },
            DnsRecord {
                record_type: "MX".to_string(),
                name: "@".to_string(),
                value: "10 mail.example.com".to_string(),
                ttl: "自动".to_string(),
            },
            DnsRecord {
                record_type: "TXT".to_string(),
                name: "@".to_string(),
                value: "\"v=spf1 include:_spf.example.com ~all\"".to_string(),
                ttl: "自动".to_string(),
            },
        ];
        info!("初始化DNS记录完成：域名数量：{}", self.dns_records.len());

        (dns_provider_list, domain_list, dns_records)
    }

    async fn handle_reload(
        connection: DatabaseConnection,
        filter: Filter,
    ) -> Result<(Vec<Account>, Vec<Domain>, u64, String), Box<dyn Error + Send>> {
        info!("开始从数据库重新加载界面数据");
        debug!("当前过滤条件: {:?}", filter);

        info!("查询账户列表");
        let list_accounts_result = list_accounts(&connection).await.unwrap_or_else(|e| {
            error!("查询账号列表发生了异常！错误详情: {}", e);
            vec![]
        });
        info!("成功获取 {} 个账户信息", list_accounts_result.len());

        let provider_account_id = filter.selected_provider.map(|provider| {
            debug!(
                "使用选定的提供商过滤: {}, ID: {}",
                provider.provider_name, provider.account_id
            );
            provider.account_id
        });

        info!("查询域名列表，账户ID过滤: {:?}", provider_account_id);
        let list_domain_result = get_account_domains(&connection, provider_account_id)
            .await
            .unwrap_or_else(|e| {
                error!("查询域名列表发生了异常！错误详情: {}", e);
                vec![]
            });
        info!("成功获取 {} 个域名信息", list_domain_result.len());

        let record: Vec<DnsRecord> = match &filter.selected_domain {
            None => {
                debug!("未选择特定域名，不加载DNS记录");
                vec![]
            }
            Some(domain) => {
                info!(
                    "查询选定域名的DNS记录: {}, ID: {:?}",
                    domain.name, domain.id
                );
                let result = get_records_by_domain(&connection, domain.id).await;
                debug!("DNS记录查询结果: {:?}", result);
                vec![]
            }
        };

        info!("查询域名总数");
        let total_count = count_all_domains(&connection).await.unwrap_or_else(|err| {
            error!("查询域名总数发生了异常！错误详情: {}", err);
            0
        });

        info!(
            "数据加载完成 - 账号: {}, 域名: {}, 总域名数: {}, DNS记录: {}",
            list_accounts_result.len(),
            list_domain_result.len(),
            total_count,
            record.len()
        );

        let domain_list = list_domain_result
            .into_iter()
            .map(|domain| {
                debug!("转换域名数据: {}", domain.domain_name);
                Domain {
                    id: None,
                    name: domain.domain_name,
                    provider: DnsProvider::Aliyun,
                    status: DomainStatus::Active,
                    expiry: "".to_string(),
                    records: vec![],
                }
            })
            .collect();

        Ok((
            list_accounts_result,
            domain_list,
            total_count,
            "".to_string(),
        ))
    }

    fn handle_reset(&mut self) {
        self.filter.reset();
        self.domain_list.clear();
        self.domain_providers.clear();
    }

    fn handle_sync(&self) -> Task<Message> {
        info!(
            "开始同步域名数据，当前提供商数量: {}",
            self.domain_providers.len()
        );
        match &self.filter.selected_provider {
            None => {
                info!("未选择特定提供商，将查询所有域名服务商的域名记录");
                if self.domain_providers.is_empty() {
                    warn!("当前没有可用的域名提供商，同步操作将返回空结果");
                }
                Task::batch(self.domain_providers.clone().into_iter().map(|provider| {
                    info!("准备同步提供商: {}", provider.provider_name);
                    Task::perform(Self::handle_domain_reload(provider), |result| {
                        let (dns_records, relative_provider) = result;
                        info!(
                            "获取提供商 {} 的DNS记录成功，共 {} 条记录",
                            relative_provider.provider_name,
                            dns_records.len()
                        );
                        Message::QueryDomainResult(dns_records, relative_provider.clone())
                    })
                }))
            }
            Some(provider) => {
                info!(
                    "查询单个域名服务商的域名记录: 「{}」",
                    &provider.provider_name
                );
                let domain_provider = provider.clone();
                Task::perform(Self::handle_domain_reload(domain_provider), |result| {
                    let (dns_records, relative_provider) = result;
                    info!(
                        "获取提供商 {} 的DNS记录成功，共 {} 条记录",
                        relative_provider.provider_name,
                        dns_records.len()
                    );
                    Message::QueryDomainResult(dns_records, relative_provider.clone())
                })
            }
        }
    }

    /// 同步域名信息
    fn handle_sync_domain(&mut self) -> Task<Message> {
        info!("开始同步域名信息，当前同步状态: {}", self.is_syncing);
        self.is_syncing = true;

        // 克隆数据库连接，因为我们需要将它移动到异步任务中
        let conn = match &self.connection {
            Some(connection) => {
                debug!("获取到有效的数据库连接");
                connection.clone()
            }
            None => {
                error!("当前没有数据库连接！同步域名操作无法继续");
                return Task::perform(
                    async { SyncResult::Failed("数据库连接失败".to_string()) },
                    |result| Message::SyncAllDomainsComplete(result),
                );
            }
        };

        let client = self.dns_client.clone();
        let conn_clone = conn.clone();

        // 同步域名信息
        Task::perform(
            async move {
                info!("开始执行域名同步任务");

                let domains = Self::sync_domains(client).await;
                info!("从DNS客户端获取到 {} 个域名", domains.len());

                if domains.is_empty() {
                    error!("没有查询到域名信息，同步任务终止");
                    return SyncResult::Failed("未获取到域名信息".to_string());
                }

                // 获取所有账户信息
                let accounts = match list_accounts(&conn_clone).await {
                    Ok(accounts) => {
                        info!("成功获取账户列表，共 {} 个账户", accounts.len());
                        accounts
                    }
                    Err(err) => {
                        error!("获取账户列表失败: {}，同步任务终止", err);
                        return SyncResult::Failed("获取账户列表失败".to_string());
                    }
                };

                // 遍历所有账户，同步域名信息
                for (index, account) in accounts.iter().enumerate() {
                    info!(
                        "开始处理第 {}/{} 个账户: {}",
                        index + 1,
                        accounts.len(),
                        account.username
                    );
                    let provider = DomainProvider::from(account.clone());
                    let (domains, _) = Self::handle_domain_reload(provider.clone()).await;

                    if domains.is_empty() {
                        info!("账户 {} 没有域名信息，跳过处理", account.username);
                        continue;
                    }

                    info!(
                        "账户 {} 有 {} 个域名，准备更新到数据库",
                        account.username,
                        domains.len()
                    );

                    // 删除该账户下的所有域名
                    if let Err(err) = delete_domain_by_account(&conn, provider.account_id).await {
                        error!(
                            "删除账户 {} 下的域名失败: {}，跳过此账户处理",
                            account.username, err
                        );
                        continue;
                    }
                    debug!("成功删除账户 {} 下的旧域名记录", account.username);

                    // 将域名添加到数据库
                    let new_domains: Vec<NewDomain> = domains
                        .into_iter()
                        .map(|domain| {
                            debug!("准备添加域名: {}", domain.name);
                            NewDomain {
                                domain_name: domain.name,
                                registration_date: None,
                                expiration_date: None,
                                registrar: None,
                                status: crate::models::domain::DomainStatus::Active,
                                account_id: provider.account_id,
                            }
                        })
                        .collect();

                    if let Err(err) = add_domain_many(&conn_clone, new_domains.clone()).await {
                        error!("添加账户 {} 的域名失败: {}", account.username, err);
                    } else {
                        info!("成功添加账户 {} 的所有域名到数据库", account.username);
                        
                        // 同步DNS记录
                        info!("开始同步账户 {} 的DNS记录", account.username);
                        for new_domain in &new_domains {
                            if let Err(err) = Self::sync_dns_records_for_domain(
                                &conn_clone, 
                                &new_domain.domain_name, 
                                account.id
                            ).await {
                                error!("同步域名 {} 的DNS记录失败: {}", new_domain.domain_name, err);
                            } else {
                                info!("成功同步域名 {} 的DNS记录", new_domain.domain_name);
                            }
                        }
                    }
                }

                info!("所有账户的域名同步完成");
                SyncResult::Success
            },
            |result| {
                info!("同步域名完成: {:?}", result);
                Message::SyncAllDomainsComplete(result)
            },
        )
    }

    /// 同步指定域名的DNS记录
    async fn sync_dns_records_for_domain(
        conn: &DatabaseConnection,
        domain_name: &str,
        account_id: i64,
    ) -> Result<(), String> {
        use crate::storage::domains::find_domain_by_name_and_account;
         
         // 查找域名实体
         let domain_entity = match find_domain_by_name_and_account(conn, domain_name, account_id).await {
             Ok(Some(domain)) => domain,
             Ok(None) => {
                 return Err(format!("未找到域名: {}", domain_name));
             }
             Err(err) => {
                 return Err(format!("查找域名失败: {}", err));
             }
         };
         
         // 创建DNS客户端 - 这里需要从现有的dns_client获取
         // 暂时跳过DNS记录同步，因为需要访问实例的dns_client
         warn!("DNS记录同步功能需要重构以访问实例的dns_client");
         return Ok(());
        
        // DNS记录同步功能暂时跳过，需要重构以访问实例的dns_client
         info!("域名 {} 的DNS记录同步已跳过，等待重构", domain_name);
        
        Ok(())
    }

    fn handle_provider_selected(&mut self, provider: Option<DomainProvider>) -> Task<Message> {
        self.filter.selected_provider = provider;
        self.in_query = true;

        // Task::perform(Self::handle_domain_reload(clone_provider), |result| {
        //     let (dns_records, relative_provider) = result;
        //
        //     info!("获取dns记录成功:{:?}", &dns_records);
        //     Message::QueryDomainResult(dns_records, relative_provider.clone())
        // })
        self.update(Message::Reload)
    }

    fn handle_add_provider_form_credential_changed(
        &mut self,
        credential: Credential,
    ) -> Task<Message> {
        self.add_domain_provider_form.credential = Some(credential);
        Task::none()
    }

    fn add_credential(&mut self) -> Task<Message> {
        info!("开始添加域名托管商凭证");
        let form_value = self.add_domain_provider_form.clone();
        debug!(
            "表单数据: 提供商名称={}, 提供商类型={:?}",
            form_value.provider_name, form_value.provider
        );

        // 参数校验
        if form_value.provider.is_none() {
            error!("提供商类型未选择");
            self.message = "请选择提供商类型".into();
            return Task::none();
        }

        if form_value.credential.is_none() {
            error!("凭证信息未提供");
            self.message = "请提供凭证信息".into();
            return Task::none();
        }

        let domain_provider = NewAccount {
            provider: form_value.provider.unwrap(),
            username: form_value.provider_name.clone(),
            email: "example@qq.com".to_string(),
            credential: form_value.credential.unwrap(),
        };

        info!(
            "添加域名托管商: {}, 类型: 「{}」",
            &domain_provider.username,
            &domain_provider.provider.name()
        );

        // 创建新增域名托管商信息
        match &mut self.connection {
            None => {
                error!("数据库连接未初始化，无法添加托管商");
                self.message = "数据库连接未初始化".into();
                Task::none()
            }
            Some(connection) => {
                info!("开始异步添加托管商到数据库");
                let conn_clone = connection.clone();
                Task::perform(
                    async move {
                        info!("执行添加托管商操作");
                        let cnn = conn_clone.clone();
                        let account = create_account(cnn, domain_provider).await;
                        account
                    },
                    |response| {
                        match response {
                            Ok(_) => info!("托管商添加成功，准备刷新界面"),
                            Err(err) => error!("托管商添加失败，错误: {:?}", err),
                        }
                        Message::Reload
                    },
                )
            }
        }
    }
}

fn domain_row(domain: &Domain, selected: bool, font: Font) -> Element<Message, StyleType> {
    let status = Text::new(domain.status.text());

    let expiry = Text::new(&domain.expiry);

    let content = Row::new()
        .spacing(10)
        .push(
            Text::new(format!(
                "{}{}\n",
                &domain.name,
                if selected { " ✓" } else { "" }
            ))
            .font(font)
            .width(Length::FillPortion(3)),
        )
        .push(Text::new(domain.provider.name()).width(Length::FillPortion(1)))
        .push(status.width(Length::FillPortion(1)))
        .push(expiry.width(Length::FillPortion(1)))

        .align_y(Alignment::Center);

    // 使用Container替代Button
    let container = Container::new(content)
        .padding(10)
        .width(Length::Fill)
        .class(if selected {
            ContainerType::Selected
        } else {
            ContainerType::Hoverable
        });

    // 使用MouseArea使Container可点击
    mouse_area(container)
        .on_press(Message::DomainSelected(domain.clone()))
        .into()
}

fn init_dns_client(config: &Config) -> Result<DnsClient, Box<dyn Error>> {
    if config.ali_access_key_id == None || config.ali_access_key_secret == None {
        // 读取环境变量里面的账号认证信息
        let access_key_id =
            env::var("ALIBABA_CLOUD_ACCESS_KEY_ID").expect("Cannot get access key id.");
        let access_key_secret =
            env::var("ALIBABA_CLOUD_ACCESS_KEY_SECRET").expect("Cannot get access key id.");
        info!("初始化客户端成功");
        Ok(DnsClient::new(
            access_key_id,
            access_key_secret,
            "cn".to_string(),
            vec![],
        ))
    } else {
        Ok(DnsClient::new(
            config.ali_access_key_id.clone().unwrap(),
            config.ali_access_key_secret.clone().unwrap(),
            "cn".to_string(),
            vec![],
        ))
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
                            last_page: Some(app.current_page.clone()),
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

// 辅助组件
fn provider_item(provider: &DomainProvider, selected: bool) -> Element<Message, StyleType> {
    let content = Row::new()
        .spacing(10)
        .push(
            // 添加图标
            Text::new("🌐").width(30).height(30),
        )
        .push(Text::new(format!("{}", provider.provider_name,)).width(Length::Fill));

    // 使用Container替代Button
    let container: Container<Message, StyleType> =
        Container::new(content).padding(10).width(Length::Fill)
        .class(if selected {
            ContainerType::Selected
        } else {
            ContainerType::Hoverable
        });

    // 使用MouseArea使Container可点击
    mouse_area(container)
        .on_press(Message::ProviderSelected(Some(provider.clone())))
        .into()
}

/// 创建统计卡片组件
///
/// # 参数
/// * `title` - 卡片标题
/// * `value` - 统计数值
/// * `description` - 描述信息
fn stat_card(title: String, value: String, description: &str) -> Element<Message, StyleType> {
    Container::new(
        Column::new()
            .spacing(8)
            .push(
                Row::new()
                    .spacing(6)
                    .push(text("📊").size(12))
                    .push(Text::new(title).size(12))
                    .align_y(Alignment::Center)
            )
            .push(
                Text::new(value)
                    .size(20)
                    .width(Length::Fill)
            )
            .push(
                Text::new(description)
                    .size(10)
                    .width(Length::Fill)
            )
            .align_x(Alignment::Start)
    )
    .padding(12)
    .width(Length::FillPortion(1))
    .class(ContainerType::Bordered)
    .into()
}

/// 创建信息行组件
///
/// # 参数
/// * `label` - 标签文本
/// * `value` - 值文本
fn info_row<'a>(label: &'a str, value: &'a str) -> Row<'a, Message, StyleType> {
    Row::new()
        .spacing(12)
        .push(
            Text::new(label)
                .size(12)
                .width(Length::Fixed(120.0))
        )
        .push(
            Text::new(value)
                .size(12)
                .width(Length::Fill)
        )
        .align_y(Alignment::Center)
        .padding([4, 0])
}

fn dns_row(record: &DnsRecord, index: usize) -> Row<Message, StyleType> {
    Row::new()
        .spacing(10)
        .push(Text::new(&record.record_type).width(60))
        .push(Text::new(&record.name).width(80))
        .push(Text::new(&record.value).width(Length::Fill))
        .push(Text::new(&record.ttl).width(60))
        .push(
            Row::new()
                .spacing(5)
                .push(button(Text::new("✎")).on_press(Message::EditDnsRecord(index)))
                .push(button(Text::new("🗑")).on_press(Message::DeleteDnsRecord(index))),
        )
}

#[cfg(test)]
mod tests {
    use crate::configs::gui_config::Config;
    use crate::get_text;
    use crate::gui::manager::DomainManager;
    use crate::gui::model::domain::{DnsProvider, DnsRecord, Domain, DomainStatus};
    use crate::gui::model::gui::ReloadModel;
    use crate::gui::pages::domain::DomainProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::gui::types::message::Message;
    use crate::storage::init_memory_database;
    use tracing_test::traced_test;

    // tests using this will require the  annotation
    #[traced_test]
    #[tokio::test]
    async fn new_instance() {
        let connection = init_memory_database()
            .await
            .expect("Cannot initialize memory database.");
        DomainManager::new(
            Config {
                ali_access_key_id: Some("12123".to_string()),
                ali_access_key_secret: Some("12123".to_string()),
                ..Default::default()
            },
            connection,
        );
        return;
    }

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
    // needed to not collide with other tests generating configs files
    fn test_correctly_update_ip_version() {
        let mut app = DomainManager::default();
        let _ = app.update(Message::AddDnsRecord);
    }

    #[test]
    // needed to not collide with other tests generating configs files
    fn test_correctly_reload_complete() {
        let mut app = DomainManager::default();

        let providers = vec![DomainProvider {
            account_id: 1,
            provider_name: "test".to_string(),
            provider: DnsProvider::Aliyun,
            credential: Credential::UsernamePassword(UsernamePasswordCredential {
                username: "test".to_string(),
                password: "pass".to_string(),
            }),
        }];

        let domains = vec![Domain {
            id: None,
            name: "test_domain".to_string(),
            provider: DnsProvider::Aliyun,
            status: DomainStatus::Active,
            expiry: "2023-12-12".to_string(),
            records: vec![],
        }];

        let records = vec![DnsRecord {
            name: "www".to_string(),
            record_type: "A".to_string(),
            value: "127.0.0.1".to_string(),
            ttl: "6000".to_string(),
        }];

        let _ = app.update(Message::ReloadComplete(ReloadModel::new_from(
            providers, domains, records, 1,
        )));

        assert_eq!(app.domain_providers.len(), 1);
        let provider = app.domain_providers.get(0);
        assert_eq!(provider.unwrap().provider_name, "test");

        assert_eq!(app.domain_list.len(), 1);
        let provider = app.domain_list.get(0);
        let domain = provider.unwrap();
        assert_eq!(domain.name, "test_domain");
        assert_eq!(domain.provider, DnsProvider::Aliyun);
        assert_eq!(domain.status, DomainStatus::Active);

        assert_eq!(app.dns_records.len(), 1);
        let record = app.dns_records.get(0);
        let record = record.unwrap();
        assert_eq!(record.name, "www");
        assert_eq!(record.record_type, "A");
        assert_eq!(record.value, "127.0.0.1");
    }
}
