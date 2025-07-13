use crate::api::ali_api::{
    add_aliyun_dns_record, delete_aliyun_dns, query_aliyun_dns_list,
    query_aliyun_dns_operation_list,
};
use crate::api::dns_client::{DnsClient, DnsClientTrait};
use crate::api::model::dns_operate::RecordLog;
use crate::api::provider::aliyun::AliyunDnsClient;
use crate::configs::gui_config::LICENCE;
use crate::gui::components::footer::footer;
use crate::gui::components::header::header;
use crate::gui::model::domain::{DnsProvider, DnsRecord, Domain, DomainStatus};
use crate::gui::model::form::{AddDnsField, AddDomainField};
use crate::gui::pages::demo::scrollables::scrollables;
use crate::gui::pages::domain::{
    add_domain_page, add_domain_provider_page, AddDomainProviderForm, DomainProvider,
};
use crate::gui::pages::domain_dns_record::{add_dns_record, dns_record};
use crate::gui::pages::help::help;
use crate::gui::pages::names::{DemoPage, Page};
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
use crate::storage::{
    add_domain, create_account, delete_domain, delete_domain_by_account, list_accounts,
    list_domains,
};
use crate::translations::types::language::Language;
use crate::translations::types::locale::Locale;
use crate::utils::types::icon::Icon;
use crate::utils::types::web_page::WebPage;
use crate::{get_text, Config, StyleType};
use chrono::Utc;
use iced::futures::future::err;
use iced::keyboard::Key;
use iced::widget::{
    button, container, horizontal_rule, horizontal_space, scrollable, Button, Column, Container,
    Row, Text, Tooltip,
};
use iced::Event::Window;
use iced::{
    keyboard, window, Alignment, Element, Font, Length, Point, Size, Subscription, Task, Theme,
};
use mockall::mock;
use sea_orm::DatabaseConnection;
use std::error::Error;
use std::sync::Mutex;
use std::{env, process};
use tracing::{debug, error, info};

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
}

#[derive(Debug, Clone)]
struct DomainStats {
    total: usize,
    expiring: usize,
    providers: usize,
}

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
        let body = match &self.current_page {
            Page::DomainPage => {
                Container::new(
                    Row::new()
                        // 左侧托管商导航
                        .push(Self::provider_sidebar(self).width(Length::Fixed(300.0)))
                        .push(self.domain_list().width(Length::FillPortion(5))) // 中间域名列表
                        .push_maybe(match &self.filter.selected_domain {
                            Some(domain) => Some(self.detail_panel().width(Length::FillPortion(2))),
                            // 右侧详情面板
                            None => Container::new(Text::new("选择域名以查看详情"))
                                .width(Length::FillPortion(2))
                                .into(),
                        }) // 右侧详情面板
                        .height(Length::Fill)
                        .width(Length::Fill),
                )
                .class(ContainerType::HighlightedOnHeader)
            }
            Page::AddDomain => add_domain_page(self),
            Page::DnsRecord => dns_record(self),
            Page::AddRecord => add_dns_record(self),
            Page::Help => help(self),
            Page::Demo(demo) => match demo {
                DemoPage::Scrollers => scrollables(self),
            },
            Page::AddProvider => add_domain_provider_page(self),
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

        // 页头
        Column::new()
            .push(header)
            .push(body.height(Length::Fill))
            .push(footer)
            .into()
    }

    // 左侧托管商导航
    fn provider_sidebar(app: &DomainManager) -> Container<Message, StyleType> {
        let provider_list = Column::new().padding(10).spacing(10).width(Length::Shrink);
        debug!("托管商数量：「{}」", app.domain_providers.len());
        let provider_list = app
            .domain_providers
            .iter()
            .fold(provider_list, |col, provider| {
                let is_selected = app.filter.selected_provider.as_ref() == Some(provider);
                col.push(
                    provider_item(provider, is_selected)
                        .on_press(Message::ProviderSelected(provider.clone())),
                )
            });

        let sidebar = Column::new()
            .push(Text::new("域名托管商").size(16))
            .push(scrollable(provider_list).height(Length::Shrink))
            .spacing(15);

        container(sidebar).height(Length::Fill).padding(10).into()
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
            .push(Text::new(title).size(20).width(Length::Fill))
            .push(
                Row::new()
                    .spacing(10)
                    .push(button("重置").on_press(Message::Reset))
                    .push(button("模拟").on_press(Message::Mock))
                    .push(
                        button("导出")
                            .on_press(Message::Export)
                            .class(ButtonType::Alert),
                    ),
            )
            .padding(10);

        // 统计卡片
        let stats = Row::new()
            .spacing(15)
            .push(stat_card(
                "总域名数".to_string(),
                self.stats.total.to_string(),
                "本月新增3个",
            ))
            .push(stat_card(
                "即将到期".to_string(),
                self.stats.expiring.to_string(),
                "30天内到期",
            ))
            .push(stat_card(
                "托管商".to_string(),
                self.stats.providers.to_string(),
                "全部正常",
            ))
            .width(Length::Fill);

        // 域名列表
        let domain_list = Column::new().spacing(5).padding(5);

        debug!("域名数量：「{}」", self.domain_list.len());
        let domain_list = self
            .domain_list
            .iter()
            .filter(|domain| match &self.filter.selected_provider {
                Some(provider) => domain.provider == provider.provider,
                None => true,
            })
            .enumerate()
            .fold(domain_list, |column, (index, domain)| {
                let is_selected = self.filter.selected_domain == Some(domain.clone());
                let button_event = if let false = is_selected {
                    debug!("当前是否添加异常信息");
                    Some(Message::DomainSelected(domain.clone()))
                } else {
                    None
                };
                column.push(
                    //SelectedDomainRowStyle
                    domain_row(domain, is_selected, font).on_press_maybe(button_event),
                )
            });

        let content = Column::new()
            .spacing(15)
            .push(header)
            .push(stats)
            .push(scrollable(domain_list).height(Length::Shrink));
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
    }

    // 右侧详情面板
    fn detail_panel(&self) -> Container<Message, StyleType> {
        if let Some(index) = &self.filter.selected_domain {
            if let Some(domain) = self.domain_list.get(0) {
                info!("选中了，现在查看详情：当前选中域名：「{:?}」", &domain.name);
                return self.domain_detail(domain);
            }
        }
        info!("没选中，查看提示信息");
        // 如果没有选择域名，显示空状态
        container(Text::new("选择域名以查看详情"))
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .into()
    }

    fn domain_detail<'a>(&'a self, domain: &'a Domain) -> Container<'a, Message, StyleType> {
        let domain_title = Row::new()
            .spacing(10)
            .push(Text::new(&domain.name).size(20))
            .push(container(Text::new(domain.provider.name())));

        let status = Text::new(domain.status.text()); //.style(domain.status.color());

        let domain_info = Column::new()
            .spacing(10)
            .push(info_row("注册日期", "2020-08-15"))
            .push(info_row("到期日期", &domain.expiry))
            .push(info_row("DNS服务器", &domain.name))
            .push(info_row("域名状态", "").push(status));

        // 托管商特色功能
        let mut features = Row::new().spacing(10);
        for feature in domain.provider.features() {
            features = features
                .push(button(feature).on_press(Message::FeatureClicked(feature.to_string())));
        }

        // DNS记录管理
        let dns_header = Row::new()
            .spacing(10)
            .push(Text::new("DNS记录管理").size(16))
            .push(horizontal_space().width(Length::Fill))
            .push(
                Button::new(Text::new(get_text("query_dns_record")))
                    .on_press(Message::AddDnsRecord),
            )
            .push(
                Button::new(Text::new(get_text("add_dns_record"))).on_press(Message::AddDnsRecord),
            );

        let dns_table = Column::new().spacing(5);

        let dns_table = self
            .dns_records
            .iter()
            .enumerate()
            .fold(dns_table, |col, (index, record)| {
                col.push(dns_row(record, index))
            });

        let status = Text::new(domain.status.text()); //.style(domain.status.color());

        let content = Column::new()
            .spacing(20)
            .push(domain_title)
            .push(status)
            .push(domain_info)
            .push(features)
            .push(horizontal_rule(2))
            .push(dns_header)
            .push(scrollable(dns_table));

        container(scrollable(content))
            .width(Length::Fixed(380.0))
            .height(Length::Fill)
            .padding(10)
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
                self.update(ReloadComplete((
                    providers,
                    domains,
                    records,
                    "创建模拟数据成功".into(),
                )))
            }

            Message::Reset => {
                self.handle_reset();
                self.update(Message::Reload)
            }
            Message::Reload => {
                debug!("界面刷新,当前选择的域名托管商");
                // 更新数据
                // TODO 这里可能会影响界面刷新，需要在异步线程里面完成
                match &self.connection {
                    None => {
                        error!("当前没有数据库连接！");
                        self.update(ReloadComplete((vec![], vec![], vec![], "".to_string())))
                    }
                    Some(connection) => {
                        // 克隆连接，因为我们需要将它移动到异步任务中
                        let conn = connection.clone();
                        Task::perform(Self::handle_reload(conn), |result| {
                            match result {
                                Ok(result) => {
                                    let (accounts, domains, message) = result;
                                    let providers = accounts
                                        .into_iter()
                                        .map(|account| account.into())
                                        .collect();

                                    return ReloadComplete((
                                        providers,
                                        vec![],
                                        vec![],
                                        "".to_string(),
                                    ));
                                }
                                Err(_) => {}
                            }
                            ReloadComplete((vec![], vec![], vec![], "".to_string()))
                        })
                    }
                }
            }
            ReloadComplete(result) => {
                info!("数据重新加载完成！，当前加载数据类型：「所有」");
                let (providers, domain, record, message) = result;
                self.domain_providers = providers;
                self.domain_list = domain;
                self.dns_records = record;
                self.stats.total = 100;
                Task::none()
            }
            Message::Started => {
                info!("Application Started!");
                let _ = self.update(Message::ChangeLocale(Locale::Chinese));
                self.update(Message::Reload)
            }
            Message::ChangeLocale(locale) => {
                Self::locale(self.locale);
                self.update(Message::LocaleChanged(locale))
            }
            Message::LocaleChanged(locale) => {
                self.locale = locale;
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
                info!("域名托管商的名称发生了变化：「{}」", &name);
                self.add_domain_provider_form.provider_name = name;
                Task::none()
            }
            Message::AddProviderFormCredentialChanged(credential) => {
                self.handle_add_provider_form_credential_changed(credential)
            }
            Message::ProviderSelected(dns_provider) => self.handle_provider_selected(dns_provider),
            Message::Export => self.handle_export(),
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
                self.dns_list = dns_list;
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
                self.domain_list
                    .push(self.add_domain_field.domain_name.clone().into());
                self.update(Message::ChangePage(Page::DomainPage))
            }
            Message::QueryDomainDnsRecord(domain_name) => {
                self.current_domain_name = Some(domain_name.clone());
                self.update(Message::ChangePage(Page::DnsRecord))
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
                self.domain_list = domain_names;
                self.in_query = false;

                match &self.connection {
                    None => Task::none(),
                    Some(conn) => {
                        let clone_connection = conn.clone();
                        let clone_provider = provider.clone();

                        Task::perform(
                            async move {
                                let _ = delete_domain_by_account(
                                    &clone_connection.clone(),
                                    clone_provider.account_id.unwrap(),
                                )
                                .await
                                .expect("执行异常");

                                let _ = add_domain(
                                    clone_connection,
                                    NewDomain {
                                        domain_name: "".to_string(),
                                        registration_date: None,
                                        expiration_date: None,
                                        registrar: None,
                                        status: crate::models::domain::DomainStatus::Active,
                                        account: Account {
                                            id: clone_provider.account_id.unwrap(),
                                            username: "".to_string(),
                                            email: "".to_string(),
                                            salt: "".to_string(),
                                            api_keys: vec![],
                                            created_at: "".to_string(),
                                            last_login: None,
                                            credential_type: "".to_string(),
                                            credential_data: "".to_string(),
                                            provider_type: "".to_string(),
                                        },
                                    },
                                )
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
            _ => {
                debug!("未处理的消息：{:?}", message);
                Task::none()
            }
        }
    }

    async fn sync_domains(app: DnsClient) -> Vec<Domain> {
        info!("同步域名信息");
        let domain_name_response = app.get_all_domain_info().await;
        match domain_name_response {
            Ok(domain_names) => {
                info!(
                    "同步域名信息成功，总共同步了「{}」条域名记录",
                    domain_names.len()
                );

                // 清空原有的列表
                info!("清空历史的域名信息:{}", domain_names.len());
            }
            Err(err) => {
                error!("获取域名异常:{}", err);
            }
        }
        vec![]
    }

    async fn handle_domain_reload(provider: DomainProvider) -> (Vec<Domain>, DomainProvider) {
        let domains: Vec<Domain> = vec![];
        info!("开始查询列表");
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
                        let aliyun_dns_client = AliyunDnsClient::new(
                            apikey_credential.api_key.clone(),
                            apikey_credential.api_secret.clone(),
                        );
                        info!("开始查询列表，使用的客户端：{:?}", &aliyun_dns_client);

                        let result = aliyun_dns_client.list_domains(0, 100).await;
                        match result {
                            Ok(domain_names) => {
                                info!("获取到了【{}】条域名记录", domain_names.len());
                                (
                                    domain_names
                                        .into_iter()
                                        .map(|domain_name| Domain {
                                            name: domain_name.name,
                                            ..Default::default()
                                        })
                                        .collect(),
                                    provider,
                                )
                            }
                            Err(_) => (vec![], provider),
                        }
                    }
                    _ => {
                        info!("认证方式错误:阿里云的认证方式应该是apiKey");
                        (vec![], provider)
                    }
                }
            }
            _ => {
                info!("当前认证方式未实现:{}", provider.provider.name());
                (vec![], provider)
            }
        }
    }

    async fn handle_dns_reload(domain_name: String) -> Vec<Record> {
        info!("查询域名信息");
        let domain_list = query_aliyun_dns_list(domain_name);
        domain_list
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

    fn theme(&self) -> Theme {
        self.theme.clone()
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
        info!("从数据库里面重新加载界面");
        // 初始化提供程序列表

        let dns_provider_list = vec![DomainProvider {
            account_id: None,
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
                name: "example.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
            },
            Domain {
                name: "example2.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
            },
            Domain {
                name: "example3.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
            },
            Domain {
                name: "example4.com".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
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
    ) -> Result<(Vec<Account>, Vec<Domain>, String), Box<dyn Error + Send>> {
        info!("从数据库里面重新加载界面");
        let list_accounts_result = list_accounts(connection.clone()).await.unwrap_or_else(|e| {
            error!("查询账号列表发生了异常！{}", e);
            vec![]
        });

        let list_domain_result = list_domains(connection).await.unwrap_or_else(|e| {
            error!("查询域名列表发生了异常！:{}", e);
            vec![]
        });

        info!(
            "查询到的账号列表数量:{},域名列表数量：「{}」",
            list_accounts_result.len(),
            list_domain_result.len()
        );

        let domain_list = list_domain_result
            .into_iter()
            .map(|domain| Domain {
                name: domain.domain_name,
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "".to_string(),
            })
            .collect();

        Ok((list_accounts_result, domain_list, "".to_string()))
    }

    fn handle_reset(&mut self) {
        self.filter.reset();
        self.domain_list.clear();
        self.domain_providers.clear();
    }

    fn handle_export(&self) -> Task<Message> {
        dbg!("正在导出数据");
        Task::none()
    }

    fn handle_sync_domain(&mut self) -> Task<Message> {
        self.is_syncing = true;
        // 同步域名信息
        Task::perform(Self::sync_domains(self.dns_client.clone()), |dns_records| {
            info!("获取dns记录成功:{:?}", dns_records);
            Message::SyncAllDomainsComplete(SyncResult::Success)
        })
    }

    fn handle_provider_selected(&mut self, provider: DomainProvider) -> Task<Message> {
        let clone_provider = provider.clone();

        self.filter.selected_provider = Some(provider);
        self.in_query = true;

        Task::perform(Self::handle_domain_reload(clone_provider), |result| {
            let (dns_records, relative_provider) = result;

            info!("获取dns记录成功:{:?}", &dns_records);
            Message::QueryDomainResult(dns_records, relative_provider.clone())
        })
    }

    fn handle_add_provider_form_credential_changed(
        &mut self,
        credential: Credential,
    ) -> Task<Message> {
        self.add_domain_provider_form.credential = Some(credential);
        Task::none()
    }

    fn add_credential(&mut self) -> Task<Message> {
        let form_value = self.add_domain_provider_form.clone();

        //todo 对参数进行校验
        let domain_provider = DomainProvider {
            account_id: None,
            provider_name: form_value.provider_name,
            provider: form_value.provider.unwrap(),
            credential: form_value.credential.unwrap(),
        };

        info!(
            "添加域名托管商:{},类型：「{}」",
            &domain_provider.provider_name,
            &domain_provider.provider.name()
        );
        // 创建新增域名托管商信息
        match &mut self.connection {
            None => {
                self.message = "数据库连接未初始化".into();
                Task::none()
            }
            Some(connection) => {
                let conn_clone = connection.clone();
                Task::perform(
                    create_account(
                        conn_clone,
                        NewAccount {
                            provider: domain_provider.provider,
                            username: domain_provider.provider_name,
                            email: "example@qq.com".to_string(),
                            credential: domain_provider.credential,
                        },
                    ),
                    |_response| Message::Reload,
                )
            }
        }
    }
}

fn domain_row(domain: &Domain, selected: bool, font: Font) -> Button<Message, StyleType> {
    let status = Text::new(domain.status.text());
    //.style(domain.status.color()

    let expiry = if matches!(domain.status, DomainStatus::Warning) {
        Text::new(&domain.expiry)
    } else {
        Text::new(&domain.expiry)
    };

    let content = Row::new()
        .spacing(10)
        .push(
            Text::new(format!(
                "{}{}",
                &domain.name,
                if selected { "[☑️]" } else { "" }
            ))
            .font(font)
            .width(Length::FillPortion(3)),
        )
        .push(Text::new(domain.provider.name()).width(Length::FillPortion(1)))
        .push(status.width(Length::FillPortion(1)))
        .push(expiry.width(Length::FillPortion(1)))
        .push(
            Row::new()
                .push(
                    button(Text::new(get_text("modify")))
                        .width(Length::Fixed(100.0))
                        .on_press(Message::DomainSelected(domain.clone())),
                )
                .push(
                    button(Text::new(get_text("delete")))
                        .width(Length::Fixed(100.0))
                        .on_press(Message::DomainSelected(domain.clone())),
                )
                .spacing(5),
        )
        .align_y(Alignment::Center);

    button(content).padding(10).width(Length::Fill)
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
fn provider_item(provider: &DomainProvider, selected: bool) -> Button<Message, StyleType> {
    let content = Row::new()
        .spacing(10)
        .push(
            Text::new(provider.provider.name().to_string())
                .width(30)
                .height(30),
        )
        .push(Text::new(format!("{}", provider.provider_name,)).width(Length::Fill));
    button(content).padding(10).width(Length::Fill)
    // .style(if selected {
    //     Button::Primary
    // } else {
    //     Button::Secondary
    // })
}

fn stat_card(title: String, value: String, description: &str) -> Element<Message, StyleType> {
    Column::new()
        .spacing(5)
        .push(Text::new(title).size(14))
        .push(Text::new(value).size(24))
        .push(Text::new(description).size(12))
        .width(Length::FillPortion(1))
        .into()
}

fn info_row<'a>(label: &'a str, value: &'a str) -> Row<'a, Message, StyleType> {
    Row::new()
        .spacing(10)
        .push(Text::new(label).width(80))
        .push(Text::new(value))
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
    use crate::gui::pages::domain::DomainProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::gui::types::message::Message;
    use crate::storage::init_memory_database;

    // tests using this will require the  annotation
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
        let _ = app.update(Message::ReloadComplete((
            vec![DomainProvider {
                account_id: None,
                provider_name: "test".to_string(),
                provider: DnsProvider::Aliyun,
                credential: Credential::UsernamePassword(UsernamePasswordCredential {
                    username: "test".to_string(),
                    password: "pass".to_string(),
                }),
            }],
            vec![Domain {
                name: "test_domain".to_string(),
                provider: DnsProvider::Aliyun,
                status: DomainStatus::Active,
                expiry: "2023-12-12".to_string(),
            }],
            vec![DnsRecord {
                name: "www".to_string(),
                record_type: "A".to_string(),
                value: "127.0.0.1".to_string(),
                ttl: "6000".to_string(),
            }],
            "".into(),
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
